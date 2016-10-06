--TEST--
Test reading data
--FILE--
<?php

use Intecture\Data;

$temp_file = tempnam(sys_get_temp_dir(), 'data.');
$fh = fopen($temp_file, "w");
fwrite($fh, '{
    "bool": true,
    "i64": -5,
    "u64": 10,
    "f64": 1.2,
    "string": "abc",
    "array": [
        123,
        "def"
    ],
    "obj": {
        "a": "b"
    }
}');
fclose($fh);

// Test open
$data = new Data($temp_file);

// Test bool
$b = $data->get('/bool');
assert($b === true);

// Test i64
$i = $data->get('/i64');
assert($i === -5);

// Test u64
$i = $data->get('/u64');
assert($i === 10);

// Test f64
$i = $data->get('/f64');
assert($i === 1.2);

// Test string
$s = $data->get('/string');
assert($s === 'abc');

// Test array
$a = $data->get('/array');
assert($a[0] === 123);
assert($a[1] === 'def');

// Test object
$o = $data->get('/obj');
assert($o->get('/a') === 'b');

unlink($temp_file);

echo 'Ok';
--EXPECT--
Ok
