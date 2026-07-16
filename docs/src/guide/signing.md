# Signing Originals

`Builder` assembles a manifest declaratively — title, intent, assertions,
ingredients — and realizes it in a single `sign()` call that embeds the
signed manifest into the image bytes and returns the result. Nothing touches
disk; no timestamp authority or other network service is contacted.

## Create intent

A brand-new original uses `Builder::create()`:

```php
use Automattic\VIP\C2PA\Builder;
use Automattic\VIP\C2PA\Signer;

$builder = Builder::create(
    'photo.jpg',
    'http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture'
);
$signed = $builder->sign($unsignedBytes, 'image/jpeg', $signer);
```

The second argument is an [IPTC DigitalSourceType](https://cv.iptc.org/newscodes/digitalsourcetype/)
URL declaring where the content came from — `digitalCapture` for a camera
photo, `trainedAlgorithmicMedia` for generative-AI output, and so on. An
unknown or empty value falls back to `digitalCapture`.

(For derivatives of an existing asset, use `Builder::edit()` instead — that's
[its own chapter](./derivatives.md).)

## Assertions

Assertions are the manifest's claims about what happened. `addAssertion()`
takes the assertion label and its data as a JSON string:

```php
$builder->addAssertion('c2pa.actions', json_encode([
    'actions' => [['action' => 'c2pa.created']],
]));
```

The extension validates that the JSON parses but otherwise passes assertions
through verbatim — any label/shape the C2PA spec (or your pipeline) defines
is fair game.

## The claim generator

The manifest records what software produced it. By default that reads
`wp-c2pa` (with the extension's version); override it to attribute the
credential to your organization or product:

```php
$builder->withGenerator('NASA');
```

An empty or whitespace-only name resets to the default.

## Signing

```php
$signed = $builder->sign($bytes, 'image/jpeg', $signer);
```

`sign()` may be called with any [`Signer`](./identities.md). The returned
string is the complete signed image — write it wherever the original came
from. The builder itself is not consumed; its accumulated state is applied
fresh on each call.

Failures throw `C2paException` with a `c2pa signing failed: …` message
(malformed assertion/ingredient JSON fails earlier, at the `add*` call, as
`invalid input: …`).
