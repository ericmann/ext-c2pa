// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) Automattic, Inc.

//! Error types for `ext-c2pa`.
//!
//! On the Rust side, every fallible operation returns a typed error enum.
//! On the PHP side, those errors surface as exception classes rooted at
//! `Automattic\VIP\C2PA\C2paException`, which itself extends
//! the built-in `\RuntimeException`.
//!
//! Add domain subclasses next to the base (load failures, runtime
//! failures, ...) — see ext-infer's `error.rs` for the worked pattern,
//! including the `From<DomainError> for PhpException` mapping.

use ext_php_rs::exception::PhpException;
use ext_php_rs::ffi::zend_class_entry;
use ext_php_rs::prelude::*;
use ext_php_rs::zend::ClassEntry;
use thiserror::Error;

/// Internal error type for fallible operations inside the extension.
///
/// Every variant maps onto the single PHP-visible [`C2paException`] (which
/// extends `\RuntimeException`). The variants exist to keep call sites
/// honest about *why* something failed; if we later want catch-by-subclass
/// granularity, split [`C2paException`] into a hierarchy and fan the
/// `From` impl out across it — the call sites won't change.
#[derive(Debug, Error)]
pub enum C2paError {
    /// Reading or validating an incoming manifest failed.
    #[error("c2pa read/validate failed: {0}")]
    Read(String),

    /// Building or signing a manifest failed.
    #[error("c2pa signing failed: {0}")]
    Sign(String),

    /// The signer configuration (cert chain / key / algorithm) is invalid.
    #[error("invalid signer configuration: {0}")]
    Signer(String),

    /// A caller-supplied argument was malformed (bad JSON, unknown alg, ...).
    #[error("invalid input: {0}")]
    Input(String),
}

impl From<C2paError> for PhpException {
    fn from(err: C2paError) -> Self {
        PhpException::from_class::<C2paException>(err.to_string())
    }
}

/// Lift a `c2pa` crate error into our read/validate variant. Sign-path call
/// sites use `.map_err(|e| C2paError::Sign(e.to_string()))` explicitly so the
/// message is categorized correctly.
impl From<c2pa::Error> for C2paError {
    fn from(err: c2pa::Error) -> Self {
        C2paError::Read(err.to_string())
    }
}

/// Base exception for all `ext-c2pa` failures. Extends
/// `\RuntimeException` so existing `catch (\RuntimeException $e)` clauses
/// continue to work.
#[php_class]
#[php(name = "Automattic\\VIP\\C2PA\\C2paException")]
#[php(extends(ce = runtime_exception_ce, stub = "\\RuntimeException"))]
#[derive(Default)]
pub struct C2paException;

// `\RuntimeException` is defined by SPL, which exposes its
// `zend_class_entry *` as a `PHPAPI` global — same convention as the
// engine's `zend_ce_*` globals. SPL is a built-in module loaded before
// user extensions, so by the time our MINIT runs this pointer is non-null.
// (`ClassEntry::try_find` would go through `EG(class_table)`, which is not
// yet initialized during MINIT.)
#[allow(non_upper_case_globals)]
unsafe extern "C" {
    static spl_ce_RuntimeException: *mut zend_class_entry;
}

/// Class-entry accessor for PHP's SPL `\RuntimeException`, used by the
/// `extends(ce = ...)` linkage on [`C2paException`].
fn runtime_exception_ce() -> &'static ClassEntry {
    // SAFETY: `spl_ce_RuntimeException` is a stable PHPAPI symbol exported
    // by any SAPI we support. It is written once during SPL's MINIT (well
    // before ours) and never reassigned, so reading it as a shared
    // `&'static` is sound. A null pointer here would mean the host PHP is
    // not SPL-enabled, which is unsupported.
    unsafe { spl_ce_RuntimeException.as_ref() }
        .expect("SPL \\RuntimeException is required (host PHP missing the SPL extension?)")
}
