--TEST--
Test Template rendering with a MapBuilder hash map
--FILE--
<?php

$temp_file = tempnam(sys_get_temp_dir(), 'template.');
$fh = fopen($temp_file, "w");
fwrite($fh, '{{#key}}{{nested}}{{/key}}');
fclose($fh);

$m2builder = new Intecture\MapBuilder();
$m2builder->insert_str("nested", "test");
$m1builder = new Intecture\MapBuilder();
$m1builder->insert_map("key", $m2builder);
$template = new Intecture\Template($temp_file);
$fd = $template->render($m1builder);
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
