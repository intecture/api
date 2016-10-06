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

PHP_METHOD(Template, __construct);
PHP_METHOD(Template, render);

typedef struct _php_template {
    zend_object std;
    Template *template;
} php_template;

void inapi_init_mapbuilder(TSRMLS_D);
void free_php_mapbuilder(void *object TSRMLS_DC);
zend_object_value create_php_mapbuilder(zend_class_entry *class_type TSRMLS_DC);

PHP_METHOD(MapBuilder, __construct);
PHP_METHOD(MapBuilder, insert_str);
PHP_METHOD(MapBuilder, insert_bool);
PHP_METHOD(MapBuilder, insert_vec);
PHP_METHOD(MapBuilder, insert_map);

typedef struct _php_mapbuilder {
    zend_object std;
    void *builder;
} php_mapbuilder;

void inapi_init_vecbuilder(TSRMLS_D);
void free_php_vecbuilder(void *object TSRMLS_DC);
zend_object_value create_php_vecbuilder(zend_class_entry *class_type TSRMLS_DC);

PHP_METHOD(VecBuilder, __construct);
PHP_METHOD(VecBuilder, push_str);
PHP_METHOD(VecBuilder, push_bool);
PHP_METHOD(VecBuilder, push_vec);
PHP_METHOD(VecBuilder, push_map);

typedef struct _php_vecbuilder {
    zend_object std;
    void *builder;
} php_vecbuilder;

#endif
