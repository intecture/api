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

bool array_is_hash(HashTable *ht TSRMLS_DC);
int build_map(HashTable *ht, MapBuilder *builder TSRMLS_DC);
int build_vec(HashTable *ht, VecBuilder *builder TSRMLS_DC);

PHP_METHOD(Template, __construct);
PHP_METHOD(Template, render);

typedef struct _php_template {
    Template *template;
    zend_object std;
} php_template;

#endif
