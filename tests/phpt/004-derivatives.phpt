--TEST--
Builder::edit re-signs a derivative as an edit chained (parentOf) to the original
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
const DIGITAL_CAPTURE = 'http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture';

$signer = Signer::selfSigned();

// 1) Sign the full-size original (Create intent).
$orig = Builder::create('orig.jpg', DIGITAL_CAPTURE);
$orig->addAssertion('c2pa.actions', json_encode(['actions' => [['action' => 'c2pa.created']]]));
$signedOriginal = $orig->sign(file_get_contents("$dir/unsigned.jpg"), 'image/jpeg', $signer);

// 2) Re-sign a generated size as an EDIT chained to the SIGNED original.
$deriv = Builder::edit('orig-300x200.jpg');
$deriv->addIngredientFromBytes(
    json_encode(['title' => 'orig.jpg', 'relationship' => 'parentOf']),
    'image/jpeg',
    $signedOriginal
);
$deriv->addAssertion('c2pa.actions', json_encode(['actions' => [['action' => 'c2pa.resized']]]));
$signedDeriv = $deriv->sign(file_get_contents("$dir/resized.jpg"), 'image/jpeg', $signer);

// 3) The derivative carries its own valid manifest...
$r = Reader::fromBytes($signedDeriv, 'image/jpeg');
echo "deriv_has_manifest: ", $r->hasManifest() ? "yes" : "no", "\n";
echo "deriv_is_valid: ", $r->isValid() ? "yes" : "no", "\n";

$s = json_decode($r->summary(), true);
echo "deriv_has_resized_action: ", in_array('c2pa.resized', $s['actions'], true) ? "yes" : "no", "\n";

// ...whose provenance points back to the original via a parentOf ingredient.
echo "ingredient_count: ", count($s['ingredients']), "\n";
echo "ingredient_relationship: ", $s['ingredients'][0]['relationship'] ?? '(none)', "\n";
echo "ingredient_title: ", $s['ingredients'][0]['title'] ?? '(none)', "\n";

// 4) And it validates as Trusted against the dev CA — the whole chain checks out.
$settings = new Settings();
$settings->withTrustAnchors(file_get_contents("$dir/dev-ca.pem"));
$rt = Reader::fromBytes($signedDeriv, 'image/jpeg', $settings);
echo "deriv_trusted_state: ", $rt->validationState(), "\n";
?>
--EXPECT--
deriv_has_manifest: yes
deriv_is_valid: yes
deriv_has_resized_action: yes
ingredient_count: 1
ingredient_relationship: parentOf
ingredient_title: orig.jpg
deriv_trusted_state: Trusted
