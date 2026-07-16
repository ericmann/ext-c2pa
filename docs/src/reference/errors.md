# Errors

Every failure throws `Automattic\VIP\C2PA\C2paException`, which extends
`\RuntimeException` — existing `catch (\RuntimeException $e)` clauses keep
working, and a namespace-wide catch is one class.

```php
use Automattic\VIP\C2PA\C2paException;
use Automattic\VIP\C2PA\Reader;

try {
    $reader = Reader::fromBytes($bytes, 'image/jpeg');
} catch (C2paException $e) {
    // corrupt asset or malformed manifest — not merely "unsigned"
}
```

## What is *not* an error

The API draws a hard line between verdicts and errors:

- **No manifest** — `Reader::fromBytes()` returns normally with
  `hasManifest() === false`.
- **Invalid manifest** — a manifest that fails validation (tampered bytes,
  bad signature) returns normally with `validationState() === "Invalid"`.

Exceptions are reserved for inputs and configuration that are actually
broken.

## Message taxonomy

Messages are prefixed by failure category:

| Prefix | Thrown by | Meaning |
|---|---|---|
| `c2pa read/validate failed: …` | `Reader::fromBytes()` | The asset is corrupt or its manifest is structurally unreadable |
| `c2pa signing failed: …` | `Builder::sign()` | Manifest assembly, ingredient embedding, or the signature itself failed |
| `invalid signer configuration: …` | `Signer::fromPem()`, `generateSelfSigned()`, `sign()` | Unsupported algorithm, empty/invalid PEM, chain–key mismatch |
| `invalid input: …` | `Builder::addAssertion()`, `addIngredientFromBytes()` | Caller-supplied JSON did not parse |

The prefixes are stable enough to branch on for logging/metrics, but prefer
structuring your code so the *call site* tells you the category — each
prefix maps 1:1 to the operation you invoked.
