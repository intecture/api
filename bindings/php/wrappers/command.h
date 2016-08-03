/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef COMMAND_H
#define COMMAND_H

#include <php.h>
#include <inapi.h>

void inapi_init_command(TSRMLS_D);
void inapi_init_command_exception(TSRMLS_D);

zend_object_value create_php_command(zend_class_entry *class_type TSRMLS_DC);
void free_php_command(void *object TSRMLS_DC);

PHP_METHOD(Command, __construct);
PHP_METHOD(Command, exec);

typedef struct _php_command {
    zend_object std;

    Command command;
} php_command;

#endif
