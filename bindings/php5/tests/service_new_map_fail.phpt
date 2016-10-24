--TEST--
Test fail while creating a Service with multiple invalid Runnables
--FILE--
<?php

use Intecture\Service;
use Intecture\ServiceException;
use Intecture\ServiceRunnable;

$runnables = array(
    'status' => new ServiceRunnable('/path/to/cmd', ServiceRunnable::COMMAND),
    '_' => 'not a ServiceRunnable'
);

try {
    $service = new Service($runnables);
} catch (ServiceException $e) {
    if ($e->getMessage() == 'Array values must be instances of Intecture\ServiceRunnable') {
        echo 'OK';
    }
}

--EXPECT--
OK
