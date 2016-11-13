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

void inapi_init_file(TSRMLS_D);
void inapi_init_file_exception(TSRMLS_D);

zend_object_value create_php_file(zend_class_entry *class_type TSRMLS_DC);
void free_php_file(void *object TSRMLS_DC);

FileOptions *parse_opts(zval *opts TSRMLS_DC);

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
    zend_object std;

    File *file;
} php_file;

#endif
