# Reading & Validating

`Reader::fromBytes()` runs the complete C2PA validation once, up front, and
caches the verdict; every accessor after that is a cheap in-memory read.

```php
use Automattic\VIP\C2PA\Reader;
use Automattic\VIP\C2PA\Settings;

$reader = Reader::fromBytes($bytes, 'image/jpeg');           // integrity only
$reader = Reader::fromBytes($bytes, 'image/jpeg', $settings); // + trust check
```

- `$bytes` — the raw image, as an ordinary PHP string. PHP strings are
  binary-safe and cross the Rust boundary byte-for-byte; never run image
  bytes through any text/encoding transformation first.
- `$mime` — the asset's MIME type (`image/jpeg`, `image/png`,
  `image/webp`, …). The `c2pa` crate dispatches its parser on this value, so
  it must match the actual bytes.
- `$settings` — optional [trust configuration](./trust.md). Without it,
  validation still verifies well-formedness and cryptographic integrity; it
  just can't assert *trust*.

## The three outcomes

| Input | Result |
|---|---|
| Image with no C2PA data | Normal return: `hasManifest()` is `false`, `validationState()` is `""` |
| Image with a manifest | Normal return: verdict in `validationState()` |
| Corrupt asset / mangled manifest | Throws `C2paException` |

Treating "no manifest" as a clean verdict rather than an exception matters in
bulk pipelines: a media library full of unsigned images is the normal case,
not a failure mode.

## Verdict accessors

```php
$reader->validationState(); // "Valid" | "Invalid" | "Trusted" | ""
$reader->isValid();         // true for Valid or Trusted
$reader->isTrusted();       // true only for Trusted
```

`Trusted` is only reachable when `Settings` carried trust anchors — see
[Trust & Verdicts](./trust.md) for exactly what each state promises.

## Two views of the manifest

**`json()`** returns the complete manifest store — every manifest in the
asset, all assertions, ingredients, signature info, and validation results —
as JSON. It is the full fidelity view, and the empty string when there was no
manifest.

**`summary()`** returns a compact, stable, UI-oriented projection of the
*active* manifest:

```json
{
  "state": "Valid",
  "has_manifest": true,
  "signer": "C2PA Test Signing Cert",
  "claim_generator": "wp-c2pa",
  "title": "photo.jpg",
  "format": "image/jpeg",
  "ai_generated": false,
  "actions": ["c2pa.created", "c2pa.resized"],
  "ingredients": [{ "title": "orig.jpg", "relationship": "parentOf" }]
}
```

Field notes:

- `signer` — the issuer from the manifest's signature certificate.
- `claim_generator` — the producing software's name (handles both claim v1
  and v2 layouts).
- `ai_generated` — `true` when any `c2pa.actions` assertion carries an IPTC
  `digitalSourceType` of `trainedAlgorithmicMedia` or
  `compositeWithTrainedAlgorithmicMedia`; this is how "Made with AI" style
  badges are driven.
- `actions` — the flat list of action names (`c2pa.created`,
  `c2pa.resized`, …) across the manifest's action assertions.
- `ingredients` — title + relationship per ingredient; a `parentOf`
  relationship is the provenance link back to a parent asset.

When there is no manifest, `summary()` still returns the same shape with
`has_manifest: false` and null/empty fields — callers can decode it
unconditionally.
