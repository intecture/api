/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT directory at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This directory may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef DATA_H
#define DATA_H

#include <php.h>
#include <inapi.h>

static zend_function_entry data_functions[2];

void inapi_init_value(TSRMLS_D);
void inapi_init_value_exception(TSRMLS_D);

zend_object_value create_php_value(zend_class_entry *class_type TSRMLS_DC);
void free_php_value(void *object TSRMLS_DC);

PHP_FUNCTION(data_open);
PHP_METHOD(Value, get);

typedef struct _php_value {
    zend_object std;

    void *value;
} php_value;

#endif
