--TEST--
Builder::create + Signer sign an unsigned image; Reader confirms the result
--SKIPIF--
<?php
if (!extension_loaded('c2pa')) {
    echo 'skip ext-c2pa not loaded';
}
?>
--FILE--
<?php
use Automattic\VIP\C2PA\Builder;
use Automattic\VIP\C2PA\Reader;
use Automattic\VIP\C2PA\Settings;
use Automattic\VIP\C2PA\Signer;

$dir = __DIR__ . '/fixtures';
$unsigned = file_get_contents("$dir/unsigned.jpg");
const DIGITAL_CAPTURE = 'http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture';

// --- Sign with the compiled-in dev credential ---
$signer = Signer::selfSigned();
echo "signer_alg: ", $signer->algorithm(), "\n";

$b = Builder::create('roundtrip.jpg', DIGITAL_CAPTURE);
$b->addAssertion('c2pa.actions', json_encode(['actions' => [['action' => 'c2pa.created']]]));
$signed = $b->sign($unsigned, 'image/jpeg', $signer);
echo "produced_bytes: ", strlen($signed) > strlen($unsigned) ? "yes" : "no", "\n";

// --- Read it back: Valid without trust ---
$r = Reader::fromBytes($signed, 'image/jpeg');
echo "has_manifest: ", $r->hasManifest() ? "yes" : "no", "\n";
echo "is_valid: ", $r->isValid() ? "yes" : "no", "\n";
echo "state: ", $r->validationState(), "\n";
$s = json_decode($r->summary(), true);
echo "title: ", $s['title'], "\n";
echo "has_created_action: ", in_array('c2pa.created', $s['actions'], true) ? "yes" : "no", "\n";

// --- Trusted against the dev CA ---
$settings = new Settings();
$settings->withTrustAnchors(file_get_contents("$dir/dev-ca.pem"));
$r = Reader::fromBytes($signed, 'image/jpeg', $settings);
echo "trusted_state: ", $r->validationState(), "\n";

// --- BYOK: fromPem with the same dev chain + key signs identically ---
$byok = Signer::fromPem(
    file_get_contents("$dir/dev-ca.pem"),
    file_get_contents("$dir/dev-key.pem"),
    'es256'
);
$b2 = Builder::create('byok.jpg', DIGITAL_CAPTURE);
$b2->addAssertion('c2pa.actions', json_encode(['actions' => [['action' => 'c2pa.created']]]));
$signed2 = $b2->sign($unsigned, 'image/jpeg', $byok);
$r2 = Reader::fromBytes($signed2, 'image/jpeg', $settings);
echo "byok_trusted_state: ", $r2->validationState(), "\n";

// --- Bad algorithm is rejected ---
try {
    Signer::fromPem('x', 'y', 'not-an-alg');
    echo "bad_alg: FAIL\n";
} catch (\Automattic\VIP\C2PA\C2paException $e) {
    echo "bad_alg_throws: yes\n";
}
?>
--EXPECT--
signer_alg: es256
produced_bytes: yes
has_manifest: yes
is_valid: yes
state: Valid
title: roundtrip.jpg
has_created_action: yes
trusted_state: Trusted
byok_trusted_state: Trusted
bad_alg_throws: yes
