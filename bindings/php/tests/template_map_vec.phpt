--TEST--
Test Template rendering with a MapBuilder vector
--FILE--
<?php

$temp_file = tempnam(sys_get_temp_dir(), 'template.');
$fh = fopen($temp_file, "w");
fwrite($fh, '{{#key}}{{.}}{{/key}}');
fclose($fh);

$vbuilder = new Intecture\VecBuilder();
$vbuilder->push_str("test");
$mbuilder = new Intecture\MapBuilder();
$mbuilder->insert_vec("key", $vbuilder);
$template = new Intecture\Template($temp_file);
$fd = $template->render_map($mbuilder);
$out_fh = fopen("php://fd/$fd", "r");
$contents = fread($out_fh, 20);

if ($contents == 'test') {
    echo 'Ok';
} else {
    echo 'Fail';
}

unlink($temp_file);
--EXPECT--
Ok
