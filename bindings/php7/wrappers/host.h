/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef HOST_H
#define HOST_H

#include <php.h>
#include <inapi.h>

PHP_METHOD(Host, __construct);
PHP_METHOD(Host, connect);
PHP_METHOD(Host, connect_endpoint);
PHP_METHOD(Host, connect_payload);
PHP_METHOD(Host, data);

typedef struct _php_host {
    Host *host;
    zval *data;
    zend_object std;
} php_host;

void unwrap_value(void *value, enum DataType dtype, zval *return_value TSRMLS_DC);
int get_check_host(zval *phost, php_host **host TSRMLS_DC);

#endif
