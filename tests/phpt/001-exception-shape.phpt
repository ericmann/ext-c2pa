--TEST--
C2paException is registered and extends RuntimeException
--SKIPIF--
<?php
if (!extension_loaded('c2pa')) {
    echo 'skip ext-c2pa not loaded';
}
?>
--FILE--
<?php
use Automattic\VIP\C2PA\C2paException;

echo "registered: ", class_exists(C2paException::class) ? "yes" : "no", "\n";
echo "extends_runtime: ",
    is_subclass_of(C2paException::class, \RuntimeException::class) ? "yes" : "no",
    "\n";
?>
--EXPECT--
registered: yes
extends_runtime: yes
