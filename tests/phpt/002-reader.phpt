--TEST--
Reader::fromBytes validates signed/unsigned images and summarizes the manifest
--SKIPIF--
<?php
if (!extension_loaded('c2pa')) {
    echo 'skip ext-c2pa not loaded';
}
?>
--FILE--
<?php
use Automattic\VIP\C2PA\Reader;
use Automattic\VIP\C2PA\Settings;

$dir = __DIR__ . '/fixtures';

// --- Unsigned image: no manifest, not valid, clean summary ---
$unsigned = file_get_contents("$dir/unsigned.jpg");
$r = Reader::fromBytes($unsigned, 'image/jpeg');
echo "unsigned_has_manifest: ", $r->hasManifest() ? "yes" : "no", "\n";
echo "unsigned_is_valid: ", $r->isValid() ? "yes" : "no", "\n";
echo "unsigned_state_empty: ", $r->validationState() === '' ? "yes" : "no", "\n";
$us = json_decode($r->summary(), true);
echo "unsigned_summary_has_manifest: ", $us['has_manifest'] ? "yes" : "no", "\n";

// --- Signed image (dev cert), no trust anchors: Valid but not Trusted ---
$signed = file_get_contents("$dir/signed.jpg");
$r = Reader::fromBytes($signed, 'image/jpeg');
echo "signed_has_manifest: ", $r->hasManifest() ? "yes" : "no", "\n";
echo "signed_is_valid: ", $r->isValid() ? "yes" : "no", "\n";
echo "signed_is_trusted_without_anchors: ", $r->isTrusted() ? "yes" : "no", "\n";
echo "signed_state: ", $r->validationState(), "\n";

$s = json_decode($r->summary(), true);
echo "summary_has_manifest: ", $s['has_manifest'] ? "yes" : "no", "\n";
echo "summary_signer_present: ", (is_string($s['signer']) && $s['signer'] !== '') ? "yes" : "no", "\n";
echo "summary_claim_generator: ", $s['claim_generator'], "\n";
echo "summary_ai_generated: ", $s['ai_generated'] ? "yes" : "no", "\n";
echo "summary_has_created_action: ", in_array('c2pa.created', $s['actions'], true) ? "yes" : "no", "\n";

// --- Signed image WITH our dev CA as a trust anchor: Trusted ---
$settings = new Settings();
$settings->withTrustAnchors(file_get_contents("$dir/dev-ca.pem"));
$r = Reader::fromBytes($signed, 'image/jpeg', $settings);
echo "trusted_state: ", $r->validationState(), "\n";
echo "trusted_is_trusted: ", $r->isTrusted() ? "yes" : "no", "\n";

// --- json() returns the full manifest store on a signed asset ---
echo "json_nonempty: ", strlen($r->json()) > 0 ? "yes" : "no", "\n";
?>
--EXPECT--
unsigned_has_manifest: no
unsigned_is_valid: no
unsigned_state_empty: yes
unsigned_summary_has_manifest: no
signed_has_manifest: yes
signed_is_valid: yes
signed_is_trusted_without_anchors: no
signed_state: Valid
summary_has_manifest: yes
summary_signer_present: yes
summary_claim_generator: wp-c2pa
summary_ai_generated: no
summary_has_created_action: yes
trusted_state: Trusted
trusted_is_trusted: yes
json_nonempty: yes
