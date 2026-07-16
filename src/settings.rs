// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) Automattic, Inc.

//! `Automattic\VIP\C2PA\Settings` — trust configuration handed to
//! [`crate::reader::Reader`] and [`crate::builder::Builder`].
//!
//! A `Settings` object holds an optional PEM bundle of trust anchors. When
//! present, validation verifies the signing chain against those anchors
//! (`ValidationState::Trusted` on success); when absent, validation still
//! reports well-formedness and cryptographic integrity (`Valid`) but does
//! not assert trust. No network is ever used — there is no trust-list fetch
//! and no remote-manifest resolution (the `c2pa` crate is built without
//! those features).

use ext_php_rs::prelude::*;

use crate::error::C2paError;

/// PHP-visible trust configuration. Cheap to clone (it only carries an
/// optional PEM string), so `Reader`/`Builder` take it by value.
#[php_class]
#[php(name = "Automattic\\VIP\\C2PA\\Settings")]
#[derive(Default, Clone)]
pub struct Settings {
    pub(crate) trust_anchors_pem: Option<String>,
}

#[php_impl]
impl Settings {
    pub fn __construct() -> Self {
        Self::default()
    }

    /// Set the PEM bundle of trusted C2PA anchors — the official C2PA trust
    /// list, or a VIP/dev anchor set. One or more concatenated PEM
    /// certificates. Passing an empty string clears any configured anchors.
    pub fn with_trust_anchors(&mut self, pem: String) {
        self.trust_anchors_pem = if pem.trim().is_empty() {
            None
        } else {
            Some(pem)
        };
    }

    /// Whether trust anchors are configured (i.e. validation will attempt to
    /// reach `Trusted`, not merely `Valid`).
    pub fn has_trust_anchors(&self) -> bool {
        self.trust_anchors_pem.is_some()
    }
}

impl Settings {
    /// Build a `c2pa::Context` from these settings. Always returns a usable
    /// context: with no anchors it is the default (no trust verification);
    /// with anchors it enables `verify.verify_trust` against them. Never
    /// touches the network.
    pub(crate) fn to_context(&self) -> Result<c2pa::Context, C2paError> {
        let mut s = c2pa::Settings::new();
        if let Some(pem) = &self.trust_anchors_pem {
            s = s
                .with_value("trust.trust_anchors", pem.clone())
                .map_err(|e| C2paError::Input(e.to_string()))?;
            s = s
                .with_value("verify.verify_trust", true)
                .map_err(|e| C2paError::Input(e.to_string()))?;
        }
        c2pa::Context::new()
            .with_settings(s)
            .map_err(|e| C2paError::Input(e.to_string()))
    }
}
