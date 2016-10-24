--TEST--
Test fail while creating a Service with no args
--FILE--
<?php

use Intecture\Service;
use Intecture\ServiceException;

try {
    $service = new Service(NULL);
} catch (ServiceException $e) {
    if ($e->getMessage() == 'The first argument must be an instance of Intecture\ServiceRunnable') {
        echo 'OK';
    }
}
--EXPECT--
OK
