--TEST--
Test fail while creating a Service with an invalid Runnable object
--FILE--
<?php

use Intecture\Service;
use Intecture\ServiceException;

try {
    $service = new Service(new stdClass());
} catch (ServiceException $e) {
    if ($e->getMessage() == 'The first argument must be an instance of Intecture\ServiceRunnable') {
        echo 'OK';
    }
}

--EXPECT--
OK
