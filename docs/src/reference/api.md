# PHP API

Everything lives in the `Automattic\VIP\C2PA` namespace. This page is the
complete surface; `stubs/c2pa.stubs.php` in the repository mirrors it for
IDEs and static analysis.

All `$bytes` parameters and byte returns are ordinary PHP strings carrying
raw binary — no encoding, no base64.

## `Settings`

Trust configuration handed to `Reader` and `Builder`. See
[Trust & Verdicts](../guide/trust.md).

```php
final class Settings
{
    public function __construct();

    /** Set the PEM bundle of trusted C2PA anchors. Empty string clears it. */
    public function withTrustAnchors(string $pem): void;

    /** Whether trust anchors are configured. */
    public function hasTrustAnchors(): bool;
}
```

## `Reader`

Reads and validates an embedded manifest from image bytes. See
[Reading & Validating](../guide/reading.md).

```php
final class Reader
{
    /**
     * Validate raw image bytes of the given MIME type. A no-manifest image
     * is not an error: hasManifest() is false, validationState() is "".
     * @throws C2paException on corrupt assets / malformed manifests
     */
    public static function fromBytes(string $bytes, string $mime, ?Settings $settings = null): Reader;

    public function hasManifest(): bool;

    /** "Valid" | "Invalid" | "Trusted" | "" (no manifest). */
    public function validationState(): string;

    /** True for "Valid" or "Trusted". */
    public function isValid(): bool;

    /** True only when the chain verified against configured trust anchors. */
    public function isTrusted(): bool;

    /** Full manifest-store JSON; "" when there was no manifest. */
    public function json(): string;

    /**
     * Compact UI summary JSON: { state, has_manifest, signer,
     * claim_generator, title, format, ai_generated, actions[],
     * ingredients[] }.
     */
    public function summary(): string;
}
```

## `Signer`

A signing credential. See [Signing Identities](../guide/identities.md).

```php
final class Signer
{
    /** Development-only ES256 credential compiled into the extension. */
    public static function selfSigned(): Signer;

    /**
     * BYOK credential. $certChainPem is leaf-first; $privateKeyPem is the
     * matching PKCS#8 key; $alg is one of
     * es256|es384|es512|ps256|ps384|ps512|ed25519.
     * @throws C2paException on unsupported alg / empty PEM
     */
    public static function fromPem(string $certChainPem, string $privateKeyPem, string $alg): Signer;

    /**
     * Generate a fresh ES256 signing identity for $org, as a JSON string
     * { "chain": <leaf+CA PEM>, "key": <PKCS#8 PEM>, "ca": <CA PEM> }.
     * Persist it and anchor "ca"; build a Signer from chain+key via
     * fromPem(). An empty $org defaults to "WordPress".
     */
    public static function generateSelfSigned(string $org): string;

    /** The signing algorithm, lowercased (e.g. "es256"). */
    public function algorithm(): string;
}
```

## `Builder`

Assembles and embeds a signed manifest into image bytes. See
[Signing Originals](../guide/signing.md) and
[Derivatives](../guide/derivatives.md).

```php
final class Builder
{
    /** New original. $sourceType is an IPTC DigitalSourceType URL. */
    public static function create(string $title, string $sourceType, ?Settings $settings = null): Builder;

    /** Edit of a parent asset. Attach the original via addIngredientFromBytes(). */
    public static function edit(string $title, ?Settings $settings = null): Builder;

    /** Override the claim-generator (producer) name; defaults to "wp-c2pa". */
    public function withGenerator(string $name): void;

    /**
     * Add an assertion; $json is the assertion data as a JSON string.
     * @throws C2paException on unparseable JSON
     */
    public function addAssertion(string $label, string $json): void;

    /**
     * Attach an ingredient (e.g. the original as parentOf) from raw bytes.
     * $ingredientJson is e.g. {"title": "...", "relationship": "parentOf"}.
     * @throws C2paException on unparseable JSON
     */
    public function addIngredientFromBytes(string $ingredientJson, string $mime, string $bytes): void;

    /**
     * Sign $bytes and return the signed image bytes.
     * @throws C2paException when manifest assembly or signing fails
     */
    public function sign(string $bytes, string $mime, Signer $signer): string;
}
```

## `C2paException`

```php
class C2paException extends \RuntimeException {}
```

The single exception type for every failure — see [Errors](./errors.md) for
the message taxonomy.
