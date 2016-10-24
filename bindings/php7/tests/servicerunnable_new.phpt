--TEST--
Test creating a ServiceRunnable
--FILE--
<?php

use Intecture\ServiceRunnable;

$runnable = new ServiceRunnable('/path/to/cmd', ServiceRunnable::COMMAND);
var_dump($runnable);
$runnable = new ServiceRunnable('mysvc', ServiceRunnable::SERVICE);
var_dump($runnable);
--EXPECT--
object(Intecture\ServiceRunnable)#1 (0) {
}
object(Intecture\ServiceRunnable)#2 (0) {
}
