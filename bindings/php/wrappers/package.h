/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef PACKAGE_H
#define PACKAGE_H

#include <php.h>
#include <inapi.h>

void inapi_init_package(TSRMLS_D);
void inapi_init_package_exception(TSRMLS_D);

zend_object_value create_php_package(zend_class_entry *class_type TSRMLS_DC);
void free_php_package(void *object TSRMLS_DC);

PHP_METHOD(Package, __construct);
PHP_METHOD(Package, is_installed);
PHP_METHOD(Package, install);
PHP_METHOD(Package, uninstall);

typedef struct _php_package {
    zend_object std;

    Package package;
} php_package;

#endif
