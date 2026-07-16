# ext-c2pa

**ext-c2pa** is a PHP-native extension for [C2PA](https://c2pa.org/) Content
Credentials: it reads and validates embedded provenance manifests, and signs
image bytes — including every derivative of an original — entirely in memory,
in-process, with no network access.

It is built in Rust on the official
[`c2pa`](https://opensource.contentauthenticity.org/docs/rust-sdk/) crate
(the Content Authenticity Initiative's Rust SDK), exposed to PHP 8.3+ via
[`ext-php-rs`](https://github.com/davidcole1340/ext-php-rs). The PHP surface
lives in the `Automattic\VIP\C2PA` namespace.

## What it does

- **Read** — `Reader::fromBytes()` validates an image's embedded manifest and
  reports a verdict (`Valid`, `Invalid`, `Trusted`, or *no manifest*), plus
  the full manifest-store JSON and a compact UI-oriented summary.
- **Sign** — `Builder::create()` embeds a signed manifest into a new original;
  `Builder::edit()` re-signs a derivative as an *edit* whose provenance chains
  back to its parent via a `parentOf` ingredient.
- **Trust** — `Settings::withTrustAnchors()` supplies a PEM anchor bundle so
  validation can distinguish *cryptographically sound* (`Valid`) from
  *chained to a trusted signer* (`Trusted`).
- **Identity** — `Signer::fromPem()` for bring-your-own-key signing, or
  `Signer::generateSelfSigned()` to mint a per-site development identity.

## What it deliberately does not do

- **No filesystem access.** Every operation takes and returns raw bytes
  (ordinary PHP strings, which are binary-safe).
- **No network access.** The `c2pa` crate is compiled without remote-manifest
  fetching or trust-list downloads, and signing never contacts a timestamp
  authority. What you configure is all there is.

That posture is what makes the extension safe inside a web request lifecycle:
each call is stateless, bounded, and self-contained.

## A 30-second taste

```php
use Automattic\VIP\C2PA\Reader;

$reader = Reader::fromBytes(file_get_contents('photo.jpg'), 'image/jpeg');

if ($reader->hasManifest() && $reader->isValid()) {
    $summary = json_decode($reader->summary(), true);
    echo "Signed by: {$summary['signer']}\n";
}
```

## Provenance

ext-c2pa is the native half of the **wp-c2pa** product — C2PA for the
WordPress Media Library — whose plugin half lives at
[github.com/ericmann/wp-c2pa](https://github.com/ericmann/wp-c2pa). The
extension is WordPress-agnostic: nothing in this book requires WordPress.

Source: [github.com/ericmann/ext-c2pa](https://github.com/ericmann/ext-c2pa).
License: GPL-2.0-or-later.
