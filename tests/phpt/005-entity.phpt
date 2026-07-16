--TEST--
generateSelfSigned + withGenerator stamp a custom signing entity (e.g. NASA)
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

// Generate a per-org dev identity.
$identity = json_decode(Signer::generateSelfSigned('NASA'), true);
echo "has_chain: ", ! empty($identity['chain']) ? "yes" : "no", "\n";
echo "has_key: ", ! empty($identity['key']) ? "yes" : "no", "\n";
echo "has_ca: ", ! empty($identity['ca']) ? "yes" : "no", "\n";

// Sign with it, stamping the producer name too.
$signer = Signer::fromPem($identity['chain'], $identity['key'], 'es256');
$b = Builder::create('nasa.jpg', DIGITAL_CAPTURE);
$b->withGenerator('NASA');
$b->addAssertion('c2pa.actions', json_encode(['actions' => [['action' => 'c2pa.created']]]));
$signed = $b->sign($unsigned, 'image/jpeg', $signer);

// Trust the generated CA -> Trusted; signer + producer read as NASA.
$settings = new Settings();
$settings->withTrustAnchors($identity['ca']);
$r = Reader::fromBytes($signed, 'image/jpeg', $settings);
echo "state: ", $r->validationState(), "\n";
$s = json_decode($r->summary(), true);
echo "signer: ", $s['signer'], "\n";
echo "producer: ", $s['claim_generator'], "\n";

// Empty org falls back without error.
$fallback = json_decode(Signer::generateSelfSigned(''), true);
echo "fallback_has_chain: ", ! empty($fallback['chain']) ? "yes" : "no", "\n";
?>
--EXPECT--
has_chain: yes
has_key: yes
has_ca: yes
state: Trusted
signer: NASA
producer: NASA
fallback_has_chain: yes
