/*
 Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef PAYLOAD_H
#define PAYLOAD_H

#include <php.h>
#include <inapi.h>

PHP_METHOD(Payload, __construct);
PHP_METHOD(Payload, build);
PHP_METHOD(Payload, run);

typedef struct _php_payload {
    Payload *payload;
    zend_object std;
} php_payload;

#endif
