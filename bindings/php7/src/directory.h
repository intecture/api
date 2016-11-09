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
    Directory *directory;
    zend_object std;
} php_directory;

#endif
