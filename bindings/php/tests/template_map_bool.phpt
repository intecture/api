--TEST--
Test Template rendering with a MapBuilder bool
--FILE--
<?php

$temp_file = tempnam(sys_get_temp_dir(), 'template.');
$fh = fopen($temp_file, "w");
fwrite($fh, '{{#key}}true{{/key}}');
fclose($fh);

$builder = new Intecture\MapBuilder();
$builder->insert_bool("key", true);
$template = new Intecture\Template($temp_file);
$fd = $template->render_map($builder);
$out_fh = fopen("php://fd/$fd", "r");
$contents = fread($out_fh, 20);

if ($contents == 'true') {
    echo 'Ok';
} else {
    echo 'Fail';
}

unlink($temp_file);
--EXPECT--
Ok
