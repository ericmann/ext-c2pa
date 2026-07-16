// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) Automattic, Inc.

//! `Automattic\VIP\C2PA\Builder` — assemble and embed a signed manifest into
//! image bytes.
//!
//! Two intents:
//!
//! - `Builder::create(title, sourceType, ?Settings)` — a new original
//!   (`c2pa.created`). `sourceType` is an IPTC DigitalSourceType URL.
//! - `Builder::edit(title, ?Settings)` — a derivative/edit of a parent asset.
//!   Add the original as a `parentOf` ingredient (see
//!   `addIngredientFromBytes`) so the derivative's provenance chains back to
//!   it — this is how every generated WordPress size stays attributable.
//!
//! `sign(bytes, mime, signer)` returns the signed image bytes. Everything is
//! in memory: no temp files, no network.

use std::io::Cursor;

use ext_php_rs::binary::Binary;
use ext_php_rs::prelude::*;
use serde_json::{json, Value};

use crate::{error::C2paError, settings::Settings, signer::Signer};

/// Builds + embeds a signed manifest into image bytes. State is accumulated
/// via `addAssertion`/`addIngredientFromBytes`, then realized in `sign`.
#[php_class]
#[php(name = "Automattic\\VIP\\C2PA\\Builder")]
pub struct Builder {
    title: String,
    intent_edit: bool,
    source_type: Option<String>,
    generator: Option<String>,
    assertions: Vec<(String, Value)>,
    ingredients: Vec<(Value, String, Vec<u8>)>, // (ingredient json, mime, bytes)
    settings: Settings,
}

#[php_impl]
impl Builder {
    /// Original capture/creation. `source_type` is an IPTC DigitalSourceType
    /// URL, e.g. `http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture`.
    pub fn create(title: String, source_type: String, settings: Option<&Settings>) -> Builder {
        Builder {
            title,
            intent_edit: false,
            source_type: Some(source_type),
            generator: None,
            assertions: vec![],
            ingredients: vec![],
            settings: settings.cloned().unwrap_or_default(),
        }
    }

    /// Edit intent — for derivatives. Pair with `addIngredientFromBytes` to
    /// attach the original as the `parentOf` ingredient.
    pub fn edit(title: String, settings: Option<&Settings>) -> Builder {
        Builder {
            title,
            intent_edit: true,
            source_type: None,
            generator: None,
            assertions: vec![],
            ingredients: vec![],
            settings: settings.cloned().unwrap_or_default(),
        }
    }

    /// Override the claim-generator (producer) name embedded in the manifest,
    /// e.g. the customer's organization. Defaults to `"wp-c2pa"`.
    pub fn with_generator(&mut self, name: String) {
        let name = name.trim();
        self.generator = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
    }

    /// Add a manifest assertion. `label` e.g. `c2pa.actions`; `json` is the
    /// assertion data as a JSON string.
    pub fn add_assertion(&mut self, label: String, json: String) -> PhpResult<()> {
        let v: Value = serde_json::from_str(&json).map_err(|e| C2paError::Input(e.to_string()))?;
        self.assertions.push((label, v));
        Ok(())
    }

    /// Add an ingredient from raw bytes — typically the original as
    /// `{"title": "...", "relationship": "parentOf"}`.
    pub fn add_ingredient_from_bytes(
        &mut self,
        ingredient_json: String,
        mime: String,
        bytes: Binary<u8>,
    ) -> PhpResult<()> {
        let v: Value =
            serde_json::from_str(&ingredient_json).map_err(|e| C2paError::Input(e.to_string()))?;
        self.ingredients.push((v, mime, bytes.to_vec()));
        Ok(())
    }

    /// Sign `bytes` (raw image of `mime`) and return the signed image bytes.
    pub fn sign(&self, bytes: Binary<u8>, mime: String, signer: &Signer) -> PhpResult<Binary<u8>> {
        let ctx = self.settings.to_context()?;

        let generator = self.generator.as_deref().unwrap_or("wp-c2pa");
        let def: c2pa::ManifestDefinition = serde_json::from_value(json!({
            "title": self.title,
            "claim_generator_info": [{
                "name": generator,
                "version": env!("CARGO_PKG_VERSION"),
            }],
        }))
        .map_err(|e| C2paError::Sign(e.to_string()))?;

        let mut b = c2pa::Builder::from_context(ctx)
            .with_definition(def)
            .map_err(|e| C2paError::Sign(e.to_string()))?;

        if self.intent_edit {
            b.set_intent(c2pa::BuilderIntent::Edit);
        } else {
            // Map the IPTC URL string to a DigitalSourceType via serde (the
            // enum's variants carry those URLs as their serialized form).
            // Fall back to DigitalCapture for an unknown/empty value.
            let dst: c2pa::DigitalSourceType = self
                .source_type
                .as_deref()
                .and_then(|s| serde_json::from_value(Value::String(s.to_string())).ok())
                .unwrap_or(c2pa::DigitalSourceType::DigitalCapture);
            b.set_intent(c2pa::BuilderIntent::Create(dst));
        }

        for (label, val) in &self.assertions {
            b.add_assertion(label, val)
                .map_err(|e| C2paError::Sign(e.to_string()))?;
        }

        for (ij, imime, ibytes) in &self.ingredients {
            let mut icur = Cursor::new(ibytes.clone());
            b.add_ingredient_from_stream(ij.to_string(), imime, &mut icur)
                .map_err(|e| C2paError::Sign(e.to_string()))?;
        }

        let boxed = signer.build()?;
        let mut src = Cursor::new(bytes.to_vec());
        let mut dest = Cursor::new(Vec::<u8>::new());
        b.sign(boxed.as_ref(), &mime, &mut src, &mut dest)
            .map_err(|e| C2paError::Sign(e.to_string()))?;

        Ok(Binary::from(dest.into_inner()))
    }
}
