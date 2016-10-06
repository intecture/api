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

void inapi_init_data(TSRMLS_D);
void inapi_init_data_exception(TSRMLS_D);

zend_object_value create_php_data(zend_class_entry *class_type TSRMLS_DC);
void free_php_data(void *object TSRMLS_DC);

void return_type(void *value, enum DataType dtype, zval *return_value TSRMLS_DC);

PHP_METHOD(Data, __construct);
PHP_METHOD(Data, get);

typedef struct _php_data {
    zend_object std;

    void *value;
} php_data;

#endif
