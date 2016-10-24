--TEST--
Test creating a command
--FILE--
<?php

$cmd = new Intecture\Command("moo");
var_dump($cmd);
--EXPECT--
object(Intecture\Command)#1 (0) {
}
