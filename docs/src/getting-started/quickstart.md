# Quickstart

A full round trip — sign an image, then validate what you signed — in one
script. Everything operates on in-memory bytes: plain PHP strings in, plain
PHP strings out.

```php
<?php

use Automattic\VIP\C2PA\Builder;
use Automattic\VIP\C2PA\Reader;
use Automattic\VIP\C2PA\Signer;

const DIGITAL_CAPTURE = 'http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture';

// 1) A signing credential. selfSigned() is a development-only identity
//    compiled into the extension — see "Signing Identities" for real ones.
$signer = Signer::selfSigned();

// 2) Sign an unsigned image as a new original.
$builder = Builder::create('photo.jpg', DIGITAL_CAPTURE);
$builder->addAssertion('c2pa.actions', json_encode([
    'actions' => [['action' => 'c2pa.created']],
]));

$unsigned = file_get_contents('photo.jpg');
$signed   = $builder->sign($unsigned, 'image/jpeg', $signer);

file_put_contents('photo-signed.jpg', $signed);

// 3) Read it back.
$reader = Reader::fromBytes($signed, 'image/jpeg');

var_dump($reader->hasManifest());     // true
var_dump($reader->validationState()); // "Valid"
var_dump($reader->isValid());         // true

// 4) The compact summary drives UI badges without JSON spelunking.
$summary = json_decode($reader->summary(), true);
// [
//   'state'           => 'Valid',
//   'has_manifest'    => true,
//   'signer'          => '...',        // certificate issuer
//   'claim_generator' => 'wp-c2pa',
//   'title'           => 'photo.jpg',
//   'format'          => 'image/jpeg',
//   'ai_generated'    => false,
//   'actions'         => ['c2pa.created'],
//   'ingredients'     => [],
// ]
```

Images with no manifest are **not** an error — `Reader::fromBytes()` returns
normally with `hasManifest() === false` and an empty `validationState()`, so
"unsigned" and "invalid" stay distinct verdicts:

```php
$reader = Reader::fromBytes($anyImage, 'image/jpeg');

match (true) {
    !$reader->hasManifest() => 'no credentials',
    $reader->isTrusted()    => 'trusted',   // chained to a configured anchor
    $reader->isValid()      => 'valid',     // cryptographically sound
    default                 => 'invalid',   // tampered or broken
};
```

Genuinely malformed input — a corrupt asset or a mangled manifest — throws
`Automattic\VIP\C2PA\C2paException` (a `\RuntimeException`).

From here:

- [Signing Originals](../guide/signing.md) — intents, assertions, generators
- [Derivatives & Provenance Chains](../guide/derivatives.md) — the reason
  this extension exists
- [Trust & Verdicts](../guide/trust.md) — turning `Valid` into `Trusted`
