--TEST--
Test building a payload
--FILE--
<?php

use Intecture\Payload;

$temp_dir = tempnam(sys_get_temp_dir(), 'payload.');
unlink($temp_dir);
mkdir($temp_dir);

$fh = fopen("$temp_dir/Makefile", "w");
fwrite($fh, "all:\n\t@touch test");
fclose($fh);

$fh = fopen("$temp_dir/payload.json", "w");
fwrite($fh, '{
    "author": "Dr. Hibbert",
    "repository": "https://github.com/dhibbz/hehehe.git",
    "language": "C"
}');
fclose($fh);

$payload = new Payload($temp_dir);
$payload->build();

assert(file_exists("$temp_dir/test"));

unlink("$temp_dir/Makefile");
unlink("$temp_dir/payload.json");
unlink("$temp_dir/test");
rmdir($temp_dir);

echo 'Ok';
--EXPECT--
Ok
