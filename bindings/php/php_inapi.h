/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef PHP_INAPI_H
#define PHP_INAPI_H

#define PHP_INAPI_EXTNAME  "inapi"
#define PHP_INAPI_EXTVER   "0.2.1"

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <php.h>
#include <inapi.h>

extern zend_module_entry inapi_module_entry;
#define phpext_inapi_ptr &inapi_module_entry;

#endif
