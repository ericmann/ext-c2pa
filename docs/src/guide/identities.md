# Signing Identities

A `Signer` is a signing credential: a certificate chain, its private key, and
an algorithm. It's inert until `Builder::sign()` uses it — construction never
contacts anything, and signing never involves a timestamp authority.

There are three ways to get one, in increasing order of seriousness.

## 1. The compiled-in dev signer

```php
$signer = Signer::selfSigned();
$signer->algorithm(); // "es256"
```

An ES256 credential baked into the extension, chaining to a throwaway dev CA.
It exists so tests, demos, and local pipelines work with zero setup.

**Never use it as a real signatory** — the private key ships in every copy of
the extension, so its signatures assert nothing.

## 2. A generated per-site identity

`Signer::generateSelfSigned()` mints a fresh ES256 identity — a root CA plus
a leaf signing certificate that meets the C2PA certificate profile — with
your organization's name as the subject, so credentials read as *you* rather
than a tool name:

```php
$identity = json_decode(Signer::generateSelfSigned('NASA'), true);
// [
//   'chain' => <leaf + CA PEM>,   // use as the Signer cert chain
//   'key'   => <PKCS#8 PEM>,      // the leaf's private key
//   'ca'    => <CA PEM>,          // add to your trust anchors
// ]

$signer = Signer::fromPem($identity['chain'], $identity['key'], 'es256');
```

Two obligations come with it:

- **Persist the result** (somewhere secret, for the key). The CA is random
  per call — regenerate and you've orphaned everything already signed.
- **Anchor the `ca`** via `Settings::withTrustAnchors()` so your own output
  validates as [`Trusted`](./trust.md).

This is a *self-signed* trust root: perfect for a site or organization
verifying its own pipeline, meaningless to third parties who haven't chosen
to anchor your CA.

## 3. Bring your own key

For a real signatory — a certificate issued by a CA that verifiers actually
anchor — load your PEM material directly:

```php
$signer = Signer::fromPem(
    file_get_contents('/etc/c2pa/chain.pem'),  // leaf-first PEM chain
    file_get_contents('/etc/c2pa/key.pem'),    // matching PKCS#8 private key
    'es256'
);
```

Requirements:

- The chain is **leaf-first** (signing cert, then intermediates/root).
- The leaf must meet the C2PA signing-certificate profile (digitalSignature
  key usage, emailProtection EKU, not a CA certificate).
- Supported algorithms: `es256`, `es384`, `es512`, `ps256`, `ps384`,
  `ps512`, `ed25519` (case-insensitive).

An unsupported algorithm or empty PEM throws `C2paException`
(`invalid signer configuration: …`) at construction; a chain/key mismatch
surfaces when signing.

The key never leaves the process — there is no signing service in the loop,
which is the point: the credential holder is the platform operator, not a
third party.
