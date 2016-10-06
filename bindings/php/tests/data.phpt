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

$data = data_open($temp_file);
var_dump($data);

unlink($temp_file);
--EXPECT--
array(7) {
  ["array"]=>
  array(2) {
    [0]=>
    int(123)
    [1]=>
    string(3) "def"
  }
  ["bool"]=>
  bool(true)
  ["f64"]=>
  float(1.2)
  ["i64"]=>
  int(-5)
  ["obj"]=>
  array(1) {
    ["a"]=>
    string(1) "b"
  }
  ["string"]=>
  string(3) "abc"
  ["u64"]=>
  int(10)
}
