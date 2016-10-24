--TEST--
Test fail while creating a ServiceRunnable with an invalid type
--FILE--
<?php

use Intecture\ServiceException;
use Intecture\ServiceRunnable;

try {
    $runnable = new ServiceRunnable('mysvc', 0);
} catch (ServiceException $e) {
    if ($e->getMessage() == 'Invalid Runnable type. Must be RUNNABLE_COMMAND or RUNNABLE_SERVICE.') {
        echo 'OK';
    } else {
        var_dump($e->getMessage());
    }
}
--EXPECT--
OK
