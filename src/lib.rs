// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) Automattic, Inc.

//! `ext-c2pa` — PHP 8.3+ native C2PA Content Credentials: read/validate
//! manifests and sign Media Library images (incl. derivatives). In-memory
//! only: no filesystem, no network.
//!
//! PHP namespace: `Automattic\VIP\C2PA`.
//!
//! Public surface:
//!
//! - `Automattic\VIP\C2PA\C2paException` — base exception
//!   (extends `\RuntimeException`)
//!
//! Grow the surface from here; ext-infer is the worked reference for
//! every convention (per-call state, refuse-direct-construction classes,
//! option parsing, stubs).

#![deny(clippy::all)]

mod builder;
mod error;
mod reader;
mod settings;
mod signer;

use ext_php_rs::prelude::*;

pub use builder::Builder;
pub use error::C2paException;
pub use reader::Reader;
pub use settings::Settings;
pub use signer::Signer;

/// PHP module entry point.
///
/// The default module name is `CARGO_PKG_NAME` (`ext-c2pa`); we
/// override it to plain `c2pa` so userland calls
/// `extension_loaded('c2pa')` — matching PHP's convention of dropping
/// the `ext-` prefix.
///
/// The order of `class::<T>()` calls is significant: child exceptions
/// reference their parent's `ClassEntry`, so parents register first.
#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .name("c2pa")
        .class::<C2paException>()
        .class::<Settings>()
        .class::<Reader>()
        .class::<Signer>()
        .class::<Builder>()
}
