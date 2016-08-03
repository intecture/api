/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef TELEMETRY_H
#define TELEMETRY_H

#include <php.h>
#include <inapi.h>

void inapi_init_telemetry(TSRMLS_D);
void inapi_init_telemetry_exception(TSRMLS_D);

zend_object_value create_php_telemetry(zend_class_entry *class_type TSRMLS_DC);
void free_php_telemetry(void *object TSRMLS_DC);

zval* telemetry_to_array(Telemetry *telemetry);
zval* fsmount_to_array(FsMount *fsmount);
zval* netif_to_array(Netif *netif);

PHP_METHOD(Telemetry, __construct);
PHP_METHOD(Telemetry, load);
PHP_METHOD(Telemetry, get);

typedef struct _php_telemetry {
    zend_object std;

    zval *telemetry;
} php_telemetry;

#endif
