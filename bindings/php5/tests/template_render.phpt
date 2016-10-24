--TEST--
Test Template rendering
--FILE--
<?php

$tpl = '{{#arr}}{{.}}{{/arr}}
{{#assoc}}{{nested}}{{/assoc}}
{{#bool}}bool{{/bool}}
{{double}}
{{long}}
{{string}}
{{^null}}null{{/null}}';

$expected = 'index
moo
bool
9.75
123
abc
null';

$temp_file = tempnam(sys_get_temp_dir(), 'template.');
$fh = fopen($temp_file, "w");
fwrite($fh, $tpl);
fclose($fh);

$template = new Intecture\Template($temp_file);
$fd = $template->render(array(
    'arr' => array('index'),
    'assoc' => array('nested' => 'moo'),
    'bool' => true,
    'double' => 9.75,
    'long' => 123,
    'string' => 'abc',
    'null' => NULL
));

$out_fh = fopen("php://fd/$fd", "r");
$contents = fread($out_fh, 32);
unlink($temp_file);

if ($contents == $expected) {
    echo 'Ok';
} else {
    echo 'Fail';
}
--EXPECT--
Ok
