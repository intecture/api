/*
 Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
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

PHP_METHOD(Command, __construct);
PHP_METHOD(Command, exec);

typedef struct _php_command {
    Command *command;
    zend_object std;
} php_command;

#endif
