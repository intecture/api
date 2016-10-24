--TEST--
Test creating a Service with a single Runnable
--FILE--
<?php

use Intecture\Service;
use Intecture\ServiceRunnable;

$runnable = new ServiceRunnable('mysvc', ServiceRunnable::SERVICE);
$service = new Service($runnable);
var_dump($service);
--EXPECT--
object(Intecture\Service)#2 (0) {
}
