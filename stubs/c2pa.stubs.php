<?php
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) Automattic, Inc.
//
// IDE / static-analysis stubs for ext-c2pa (PHP namespace Automattic\VIP\C2PA).
// Hand-maintained to mirror src/*.rs; regenerate-by-hand when the surface
// changes. (cargo php stubs can also produce these once cargo-php is present.)

namespace Automattic\VIP\C2PA;

/**
 * Trust configuration handed to Reader/Builder. With trust anchors set,
 * validation can reach "Trusted"; without, it reports "Valid"/"Invalid".
 * Never touches the network.
 */
class Settings {
	public function __construct() {}

	/** Set the PEM bundle of trusted C2PA anchors. Empty string clears it. */
	public function withTrustAnchors( string $pem ): void {}

	/** Whether trust anchors are configured. */
	public function hasTrustAnchors(): bool {}
}

/**
 * Reads + validates an incoming C2PA manifest from in-memory image bytes.
 */
class Reader {
	/**
	 * Validate raw image bytes of the given MIME type. A no-manifest image is
	 * not an error: hasManifest() is false and validationState() is "".
	 */
	public static function fromBytes( string $bytes, string $mime, ?Settings $settings = null ): Reader {}

	public function hasManifest(): bool {}

	/** "Valid" | "Invalid" | "Trusted" | "" (no manifest). */
	public function validationState(): string {}

	/** True for "Valid" or "Trusted". */
	public function isValid(): bool {}

	/** True only when the chain verified against configured trust anchors. */
	public function isTrusted(): bool {}

	/** Full manifest-store JSON; "" when there was no manifest. */
	public function json(): string {}

	/**
	 * Compact UI summary JSON: { state, has_manifest, signer, claim_generator,
	 * title, format, ai_generated, actions[], ingredients[] }.
	 */
	public function summary(): string {}
}

/**
 * A C2PA signing credential. selfSigned() for dev/demo; fromPem() for BYOK.
 */
class Signer {
	/** Development-only ES256 credential compiled into the extension. */
	public static function selfSigned(): Signer {}

	/**
	 * BYOK credential. $alg is one of
	 * es256|es384|es512|ps256|ps384|ps512|ed25519.
	 */
	public static function fromPem( string $certChainPem, string $privateKeyPem, string $alg ): Signer {}

	/**
	 * Generate a fresh ES256 signing identity for $org, as a JSON string
	 * { "chain": <leaf+CA PEM>, "key": <PKCS#8 PEM>, "ca": <CA PEM> }. Persist
	 * it and trust "ca"; build a Signer from chain+key via fromPem().
	 */
	public static function generateSelfSigned( string $org ): string {}

	/** The signing algorithm, lowercased (e.g. "es256"). */
	public function algorithm(): string {}
}

/**
 * Assembles and embeds a signed manifest into image bytes.
 */
class Builder {
	/** New original. $sourceType is an IPTC DigitalSourceType URL. */
	public static function create( string $title, string $sourceType, ?Settings $settings = null ): Builder {}

	/** Edit of a parent asset (derivatives). Attach the original via addIngredientFromBytes. */
	public static function edit( string $title, ?Settings $settings = null ): Builder {}

	/** Override the claim-generator (producer) name; defaults to "wp-c2pa". */
	public function withGenerator( string $name ): void {}

	/** Add an assertion; $json is the assertion data as a JSON string. */
	public function addAssertion( string $label, string $json ): void {}

	/** Attach an ingredient (e.g. the original as parentOf) from raw bytes. */
	public function addIngredientFromBytes( string $ingredientJson, string $mime, string $bytes ): void {}

	/** Sign $bytes and return the signed image bytes. */
	public function sign( string $bytes, string $mime, Signer $signer ): string {}
}

/** Base exception; extends \RuntimeException. */
class C2paException extends \RuntimeException {}
