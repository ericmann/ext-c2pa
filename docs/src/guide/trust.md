# Trust & Verdicts

C2PA validation answers two different questions, and the API keeps them
separate:

1. **Integrity** — is the manifest well-formed and cryptographically sound?
   Has the image been altered since signing?
2. **Trust** — was it signed by someone I recognize?

Integrity is intrinsic to the bytes. Trust requires you to say who you trust,
and that's what `Settings` is for.

## Configuring anchors

```php
use Automattic\VIP\C2PA\Reader;
use Automattic\VIP\C2PA\Settings;

$settings = new Settings();
$settings->withTrustAnchors(file_get_contents('/etc/c2pa/anchors.pem'));
$settings->hasTrustAnchors();   // true

$reader = Reader::fromBytes($bytes, 'image/jpeg', $settings);
```

The anchor bundle is one or more concatenated PEM certificates — the official
C2PA trust list export, your organization's CA, a dev CA, or any mix.
Passing an empty string clears the anchors. `Settings` can also be handed to
`Builder::create()`/`Builder::edit()` to apply the same configuration on the
signing path.

There is **no implicit trust list**: the extension never fetches one from the
network, and with no anchors configured nothing is trusted. What you provide
is the entire trust universe.

## The verdict ladder

| `validationState()` | Meaning |
|---|---|
| `""` | No manifest at all (`hasManifest()` is `false`) |
| `"Invalid"` | Manifest present but broken: tampered content, bad signature, malformed claim |
| `"Valid"` | Cryptographically sound; signer chain **not** verified against anchors |
| `"Trusted"` | Sound **and** the signing chain verifies to a configured anchor |

`isValid()` is true for `Valid` *or* `Trusted`; `isTrusted()` only for
`Trusted`. Without anchors, `Trusted` is unreachable and `Valid` is the
ceiling.

## Choosing what a badge means

For a UI, the useful mapping is usually:

- `Trusted` → full credential badge, named signer
- `Valid` → "has credentials" (sound, but from a signer you haven't
  anchored)
- `Invalid` → warning — a broken credential is a stronger signal than no
  credential
- no manifest → nothing to show

Anchoring your own signing CA (see
[Signing Identities](./identities.md)) makes your own pipeline's output
validate as `Trusted` end-to-end — originals and every derivative.
