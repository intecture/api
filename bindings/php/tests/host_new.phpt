--TEST--
Test creating a host
--FILE--
<?php

$host = new Intecture\Host();
var_dump($host);

--EXPECT--
object(Intecture\Host)#1 (0) {
}
