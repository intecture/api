--TEST--
Test creating a Service with multiple Runnables
--FILE--
<?php

use Intecture\Service;
use Intecture\ServiceRunnable;

$runnables = array(
    'status' => new ServiceRunnable('/path/to/cmd', ServiceRunnable::COMMAND),
    '_' => new ServiceRunnable('mysvc', ServiceRunnable::SERVICE)
);
$service = new Service($runnables, [
    "this" => "that"
]);
var_dump($service);
--EXPECT--
object(Intecture\Service)#3 (0) {
}
