--TEST--
Test reading data
--FILE--
<?php

use Intecture\Value;

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
$value = Intecture\data_open($temp_file);

// Test bool
$bool = $value->get(Value::BOOL, '/bool');
assert($bool === true);

// Test i64
$i = $value->get(Value::INT, '/i64');
assert($i === -5);

// Test u64
$i = $value->get(Value::INT, '/u64');
assert($i === 10);

// Test f64
$i = $value->get(Value::DOUBLE, '/f64');
assert($i === 1.2);

// Test string
$s = $value->get(Value::STRING, '/string');
assert($s === 'abc');

// Test array
$a = $value->get(Value::ARR, '/array');
assert($a[0]->get(Value::INT) === 123);
assert($a[1]->get(Value::STRING) === 'def');

// Test object
$v = $value->get(Value::OBJECT, '/obj');
assert($v->get(VALUE::STRING, '/a') === 'b');

unlink($temp_file);

echo 'Ok';
--EXPECT--
Ok
