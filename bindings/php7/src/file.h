/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef FILE_H
#define FILE_H

/* File Options */
#define OPT_BACKUP_EXISTING 11
#define OPT_CHUNK_SIZE 12

#include <php.h>
#include <inapi.h>

int parse_opts(zval *opts, FileOptions *fopts TSRMLS_DC);

PHP_METHOD(File, __construct);
PHP_METHOD(File, exists);
PHP_METHOD(File, upload);
PHP_METHOD(File, upload_file);
PHP_METHOD(File, delete);
PHP_METHOD(File, mv);
PHP_METHOD(File, copy);
PHP_METHOD(File, get_owner);
PHP_METHOD(File, set_owner);
PHP_METHOD(File, get_mode);
PHP_METHOD(File, set_mode);

typedef struct _php_file {
    File *file;
    zend_object std;
} php_file;

#endif
