// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) Automattic, Inc.

//! `Automattic\VIP\C2PA\Reader` — read + validate an incoming C2PA manifest
//! from in-memory image bytes.
//!
//! `fromBytes` runs the full validation once and caches the verdict; the
//! accessors are then cheap. The image never touches disk or the network:
//! the bytes are validated in a `Cursor` and dropped.

use std::io::Cursor;

use ext_php_rs::binary::Binary;
use ext_php_rs::prelude::*;
use serde_json::{json, Value};

use crate::{error::C2paError, settings::Settings};

/// Reads + validates an incoming C2PA manifest from image bytes.
#[php_class]
#[php(name = "Automattic\\VIP\\C2PA\\Reader")]
pub struct Reader {
    json: String,
    state: String, // Debug of c2pa::ValidationState: "Valid" | "Invalid" | "Trusted"
    has_manifest: bool,
}

#[php_impl]
impl Reader {
    /// Validate `bytes` (a raw binary image string) of the given `mime`
    /// (e.g. `"image/jpeg"`). When `settings` carries trust anchors,
    /// validation can reach `Trusted`; otherwise it reports `Valid`/`Invalid`.
    ///
    /// An image with no embedded manifest is **not** an error: it returns a
    /// `Reader` with `hasManifest() === false` and `validationState()` empty.
    pub fn from_bytes(
        bytes: Binary<u8>,
        mime: String,
        settings: Option<&Settings>,
    ) -> PhpResult<Reader> {
        let buf: Vec<u8> = bytes.to_vec();
        let mut cur = Cursor::new(buf);

        // Always go through an explicit Context (the bare `Reader::from_stream`
        // is deprecated — it relies on thread-local settings). With no
        // user settings we still build a default context.
        let ctx = match settings {
            Some(s) => s.to_context()?,
            None => c2pa::Context::new(),
        };

        let reader = match c2pa::Reader::from_context(ctx).with_stream(&mime, &mut cur) {
            Ok(r) => r,
            Err(e) => {
                // A genuinely unreadable/corrupt asset (or one with a
                // malformed manifest) surfaces as an error here. Treat "no
                // C2PA data at all" as a clean no-manifest verdict rather
                // than throwing, so the plugin can record "none" uniformly.
                if is_no_manifest(&e) {
                    return Ok(Reader {
                        json: String::new(),
                        state: String::new(),
                        has_manifest: false,
                    });
                }
                return Err(C2paError::Read(e.to_string()).into());
            }
        };

        let has_manifest = reader.active_manifest().is_some();
        let state = format!("{:?}", reader.validation_state());
        Ok(Reader {
            json: reader.json(),
            state,
            has_manifest,
        })
    }

    /// Whether the asset carried an embedded C2PA manifest.
    pub fn has_manifest(&self) -> bool {
        self.has_manifest
    }

    /// The validation verdict: `"Valid"`, `"Invalid"`, `"Trusted"`, or `""`
    /// when there was no manifest.
    pub fn validation_state(&self) -> String {
        self.state.clone()
    }

    /// True when the manifest is cryptographically sound — `Valid` (integrity
    /// only) or `Trusted` (chains to a configured trust anchor).
    pub fn is_valid(&self) -> bool {
        matches!(self.state.as_str(), "Valid" | "Trusted")
    }

    /// True only when the signing chain verified against configured trust
    /// anchors. Always false when `Settings` carried no anchors.
    pub fn is_trusted(&self) -> bool {
        self.state == "Trusted"
    }

    /// The full manifest-store JSON (every manifest, assertion, ingredient,
    /// and the validation results). Empty string when there was no manifest.
    pub fn json(&self) -> String {
        self.json.clone()
    }

    /// A compact, UI-oriented summary of the active manifest as a JSON string:
    /// `{ state, has_manifest, signer, claim_generator, title, format,
    /// ai_generated, actions: [], ingredients: [{title, relationship}] }`.
    /// Parsed from the manifest store so the plugin doesn't reimplement the
    /// shape in PHP. Returns the no-manifest shape when empty.
    pub fn summary(&self) -> String {
        self.build_summary().to_string()
    }
}

impl Reader {
    fn build_summary(&self) -> Value {
        let base = json!({
            "state": self.state,
            "has_manifest": self.has_manifest,
            "signer": Value::Null,
            "claim_generator": Value::Null,
            "title": Value::Null,
            "format": Value::Null,
            "ai_generated": false,
            "actions": [],
            "ingredients": [],
        });

        if !self.has_manifest {
            return base;
        }

        let store: Value = match serde_json::from_str(&self.json) {
            Ok(v) => v,
            Err(_) => return base,
        };

        // Resolve the active manifest object.
        let active_id = store.get("active_manifest").and_then(Value::as_str);
        let manifest = active_id
            .and_then(|id| store.get("manifests").and_then(|m| m.get(id)))
            .or_else(|| {
                // Fall back to the first manifest if active id is absent.
                store
                    .get("manifests")
                    .and_then(Value::as_object)
                    .and_then(|m| m.values().next())
            });

        let manifest = match manifest {
            Some(m) => m,
            None => return base,
        };

        let signer = manifest
            .pointer("/signature_info/issuer")
            .and_then(Value::as_str)
            .map(str::to_string);

        let claim_generator = manifest
            .pointer("/claim_generator_info/0/name")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                manifest
                    .get("claim_generator")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });

        let title = manifest
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string);
        let format = manifest
            .get("format")
            .and_then(Value::as_str)
            .map(str::to_string);

        // Collect action labels and detect AI-generated provenance from any
        // `c2pa.actions` assertion's digitalSourceType.
        let mut actions: Vec<String> = Vec::new();
        let mut ai_generated = false;
        if let Some(assertions) = manifest.get("assertions").and_then(Value::as_array) {
            for a in assertions {
                // Claim v1 labels the assertion `c2pa.actions`; claim v2 uses
                // `c2pa.actions.v2`. Match the family by prefix.
                let label = a.get("label").and_then(Value::as_str).unwrap_or_default();
                if !label.starts_with("c2pa.actions") {
                    continue;
                }
                if let Some(list) = a.pointer("/data/actions").and_then(Value::as_array) {
                    for act in list {
                        if let Some(name) = act.get("action").and_then(Value::as_str) {
                            actions.push(name.to_string());
                        }
                        if let Some(dst) = act.get("digitalSourceType").and_then(Value::as_str) {
                            if dst.contains("trainedAlgorithmicMedia")
                                || dst.contains("compositeWithTrainedAlgorithmicMedia")
                            {
                                ai_generated = true;
                            }
                        }
                    }
                }
            }
        }

        let ingredients: Vec<Value> = manifest
            .get("ingredients")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .map(|i| {
                        json!({
                            "title": i.get("title").and_then(Value::as_str),
                            "relationship": i.get("relationship").and_then(Value::as_str),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        json!({
            "state": self.state,
            "has_manifest": true,
            "signer": signer,
            "claim_generator": claim_generator,
            "title": title,
            "format": format,
            "ai_generated": ai_generated,
            "actions": actions,
            "ingredients": ingredients,
        })
    }
}

/// Heuristic: did `c2pa` fail because the asset simply has no manifest, as
/// opposed to a real validation/parse error we should surface? The crate
/// returns `Error::JumbfNotFound` for assets with no C2PA data.
fn is_no_manifest(e: &c2pa::Error) -> bool {
    matches!(e, c2pa::Error::JumbfNotFound)
}
