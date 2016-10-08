/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef TEMPLATE_H
#define TEMPLATE_H

#include <php.h>
#include <inapi.h>

void inapi_init_template(TSRMLS_D);
void inapi_init_template_exception(TSRMLS_D);
void free_php_template(void *object TSRMLS_DC);
zend_object_value create_php_template(zend_class_entry *class_type TSRMLS_DC);

bool array_is_hash(HashTable *arr_hash TSRMLS_DC);
int build_map(HashTable *arr_hash, MapBuilder *builder TSRMLS_DC);
int build_vec(HashTable *arr_hash, VecBuilder *builder TSRMLS_DC);

PHP_METHOD(Template, __construct);
PHP_METHOD(Template, render);

typedef struct _php_template {
    zend_object std;
    Template *template;
} php_template;

#endif
