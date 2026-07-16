// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) Automattic, Inc.

//! `Automattic\VIP\C2PA\Signer` — the credential used to sign image bytes.
//!
//! Two ways to obtain one:
//!
//! - `Signer::selfSigned()` — a **development-only** ES256 credential whose
//!   cert chain and private key are compiled into the extension. Useful for
//!   the demo and tests; it chains to a throwaway dev CA, not a public trust
//!   list. Never ship it as a real signatory.
//! - `Signer::fromPem($certChainPem, $privateKeyPem, $alg)` — BYOK. The
//!   customer is the signatory (e.g. NASA signs with its own cert); nothing
//!   leaves the platform.
//!
//! The signer is materialized lazily (`build`) at sign time rather than held
//! as a trait object across the PHP boundary — it just carries PEM bytes and
//! an algorithm.

use std::str::FromStr;

use c2pa::create_signer::from_keys;
use c2pa::{BoxedSigner, SigningAlg};
use ext_php_rs::prelude::*;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose, PKCS_ECDSA_P256_SHA256,
};
use serde_json::json;

use crate::error::C2paError;

/// Development-only signing credential, compiled in. ES256, chains to a
/// throwaway dev CA generated for this repo. DEV/DEMO USE ONLY.
const DEV_CHAIN_PEM: &[u8] = include_bytes!("dev/dev_signer_chain.pem");
const DEV_KEY_PEM: &[u8] = include_bytes!("dev/dev_signer_key.pem");

/// A C2PA signing credential. Holds PEM bytes + algorithm; the actual
/// `c2pa` signer is built on demand by [`Signer::build`].
#[php_class]
#[php(name = "Automattic\\VIP\\C2PA\\Signer")]
#[derive(Clone)]
pub struct Signer {
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    alg: SigningAlg,
}

#[php_impl]
impl Signer {
    /// Development/demo credential compiled into the extension (ES256). Chains
    /// to a throwaway dev CA — fine for local validation and the demo, never
    /// for production signing.
    pub fn self_signed() -> Signer {
        Signer {
            cert_pem: DEV_CHAIN_PEM.to_vec(),
            key_pem: DEV_KEY_PEM.to_vec(),
            alg: SigningAlg::Es256,
        }
    }

    /// BYOK credential. `cert_chain_pem` is the leaf-first PEM chain;
    /// `private_key_pem` the matching PKCS#8 key; `alg` one of
    /// `es256|es384|es512|ps256|ps384|ps512|ed25519`.
    pub fn from_pem(
        cert_chain_pem: String,
        private_key_pem: String,
        alg: String,
    ) -> PhpResult<Signer> {
        let alg = SigningAlg::from_str(alg.trim().to_ascii_lowercase().as_str())
            .map_err(|_| C2paError::Signer(format!("unsupported signing algorithm: {alg}")))?;
        if cert_chain_pem.trim().is_empty() || private_key_pem.trim().is_empty() {
            return Err(C2paError::Signer("empty cert chain or private key".into()).into());
        }
        Ok(Signer {
            cert_pem: cert_chain_pem.into_bytes(),
            key_pem: private_key_pem.into_bytes(),
            alg,
        })
    }

    /// Generate a fresh, self-contained ES256 signing identity for the given
    /// organization, returned as a JSON string:
    /// `{ "chain": <leaf+CA PEM>, "key": <PKCS#8 PEM>, "ca": <CA PEM> }`.
    ///
    /// The credential reads as `org` (e.g. "NASA") rather than the built-in
    /// dev name. The caller should **persist** the result (the CA is random
    /// per call) and add `ca` to its trust anchors so the output validates as
    /// `Trusted`. Build a usable `Signer` from `chain` + `key` via
    /// `fromPem(chain, key, "es256")`.
    pub fn generate_self_signed(org: String) -> PhpResult<String> {
        let org = if org.trim().is_empty() {
            "WordPress".to_string()
        } else {
            org
        };
        let (chain, key, ca) = gen_dev_identity(&org)?;
        Ok(json!({ "chain": chain, "key": key, "ca": ca }).to_string())
    }

    /// The signing algorithm, lowercased (e.g. `"es256"`).
    pub fn algorithm(&self) -> String {
        self.alg.to_string()
    }
}

/// Build a CA→leaf ES256 chain whose subject organization is `org`, meeting
/// the C2PA signing-certificate profile (KU digitalSignature, EKU
/// emailProtection, SKI + AKI; leaf is not a CA). Returns
/// `(chain_pem_leaf_first, leaf_key_pkcs8_pem, ca_pem)`.
fn gen_dev_identity(org: &str) -> Result<(String, String, String), C2paError> {
    let map_err = |e: rcgen::Error| C2paError::Signer(format!("certificate generation: {e}"));

    let dn = |cn: &str| -> DistinguishedName {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::OrganizationName, org);
        dn.push(DnType::CommonName, cn);
        dn
    };

    // Root CA (self-signed).
    let ca_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).map_err(map_err)?;
    let mut ca = CertificateParams::new(Vec::<String>::new()).map_err(map_err)?;
    ca.distinguished_name = dn(&format!("{org} Content Credentials Root CA"));
    ca.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    let ca_cert = ca.self_signed(&ca_key).map_err(map_err)?;

    // Leaf signing certificate (issued by the CA — C2PA rejects self-signed
    // leaves). digitalSignature + emailProtection + AKI back to the CA.
    let leaf_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).map_err(map_err)?;
    let mut leaf = CertificateParams::new(Vec::<String>::new()).map_err(map_err)?;
    leaf.distinguished_name = dn(&format!("{org} Content Credentials"));
    leaf.is_ca = IsCa::NoCa;
    leaf.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf.extended_key_usages = vec![ExtendedKeyUsagePurpose::EmailProtection];
    leaf.use_authority_key_identifier_extension = true;
    let issuer = Issuer::from_params(&ca, &ca_key);
    let leaf_cert = leaf.signed_by(&leaf_key, &issuer).map_err(map_err)?;

    let chain = format!("{}{}", leaf_cert.pem(), ca_cert.pem());
    Ok((chain, leaf_key.serialize_pem(), ca_cert.pem()))
}

impl Signer {
    /// Materialize the `c2pa` signer for a single sign operation. No network:
    /// `tsa_url` is `None`, so no timestamp authority is contacted.
    pub(crate) fn build(&self) -> Result<BoxedSigner, C2paError> {
        from_keys(&self.cert_pem, &self.key_pem, self.alg, None)
            .map_err(|e| C2paError::Signer(e.to_string()))
    }
}
