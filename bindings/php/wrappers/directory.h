/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT directory at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This directory may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef DIRECTORY_H
#define DIRECTORY_H

/* Directory Options */
#define OPT_DO_RECURSIVE 31

#include <php.h>
#include <inapi.h>

void inapi_init_directory(TSRMLS_D);
void inapi_init_directory_exception(TSRMLS_D);

zend_object_value create_php_directory(zend_class_entry *class_type TSRMLS_DC);
void free_php_directory(void *object TSRMLS_DC);

PHP_METHOD(Directory, __construct);
PHP_METHOD(Directory, exists);
PHP_METHOD(Directory, create);
PHP_METHOD(Directory, delete);
PHP_METHOD(Directory, mv);
PHP_METHOD(Directory, get_owner);
PHP_METHOD(Directory, set_owner);
PHP_METHOD(Directory, get_mode);
PHP_METHOD(Directory, set_mode);

typedef struct _php_directory {
    zend_object std;

    Directory directory;
} php_directory;

#endif
