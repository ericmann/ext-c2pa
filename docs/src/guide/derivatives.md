# Derivatives & Provenance Chains

This is the headline feature. Real pipelines rarely serve the original file:
a CMS generates thumbnails, crops, and responsive sizes, and *those* are what
end up in pages. If only the original is signed, the provenance evaporates at
render time — every derivative is just an unsigned image.

ext-c2pa closes that gap: each derivative is re-signed as an **edit** whose
manifest carries the original as a `parentOf` ingredient. The derivative then
holds its own valid credential *and* a verifiable chain back to what it was
made from.

## The pattern

```php
use Automattic\VIP\C2PA\Builder;
use Automattic\VIP\C2PA\Reader;
use Automattic\VIP\C2PA\Signer;

$signer = Signer::selfSigned(); // dev only — see "Signing Identities"

// 1) Sign the full-size original (Create intent).
$orig = Builder::create('photo.jpg', 'http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture');
$orig->addAssertion('c2pa.actions', json_encode(['actions' => [['action' => 'c2pa.created']]]));
$signedOriginal = $orig->sign($originalBytes, 'image/jpeg', $signer);

// 2) Re-sign each generated size as an EDIT chained to the SIGNED original.
$deriv = Builder::edit('photo-300x200.jpg');
$deriv->addIngredientFromBytes(
    json_encode(['title' => 'photo.jpg', 'relationship' => 'parentOf']),
    'image/jpeg',
    $signedOriginal          // the parent's *signed* bytes
);
$deriv->addAssertion('c2pa.actions', json_encode(['actions' => [['action' => 'c2pa.resized']]]));
$signedDerivative = $deriv->sign($resizedBytes, 'image/jpeg', $signer);
```

Three details carry the meaning:

1. **`Builder::edit()`** — the edit intent takes no DigitalSourceType; this
   manifest describes a transformation, not an origin.
2. **`addIngredientFromBytes()` with `relationship: parentOf`** — pass the
   parent's **signed** bytes, so the ingredient captures the parent's own
   manifest and the chain is cryptographically anchored, not just a title
   reference.
3. **A `c2pa.resized` action** — says *what* the edit was. Use whichever
   [C2PA action](https://spec.c2pa.org/specifications/specifications/2.2/specs/C2PA_Specification.html)
   fits the transformation (`c2pa.cropped`, `c2pa.color_adjustments`, …).

## What validation sees

```php
$r = Reader::fromBytes($signedDerivative, 'image/jpeg');
$s = json_decode($r->summary(), true);

$r->isValid();          // true — the derivative's own manifest verifies
$s['actions'];          // ['c2pa.resized']
$s['ingredients'][0];   // ['title' => 'photo.jpg', 'relationship' => 'parentOf']
```

And with the signing CA configured as a trust anchor, the whole chain
verifies to `Trusted` — original and every size (this is exactly what the
test suite asserts, and what `c2patool` independently confirms against the
same files).

## Deeper chains

An edit of an edit works the same way: sign the crop with the resized
version's signed bytes as its `parentOf` ingredient. Each manifest records
one hop; the chain is the concatenation of hops, walkable through
`json()`'s full ingredient data.
