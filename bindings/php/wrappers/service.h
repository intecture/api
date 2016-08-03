/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#ifndef SERVICE_H
#define SERVICE_H

/* Runnable Options */
#define RUNNABLE_COMMAND 21
#define RUNNABLE_SERVICE 22

#include <php.h>
#include <inapi.h>

void inapi_init_service(TSRMLS_D);
void inapi_init_service_exception(TSRMLS_D);
void inapi_init_service_runnable(TSRMLS_D);

zend_object_value create_php_service(zend_class_entry *class_type TSRMLS_DC);
zend_object_value create_php_service_runnable(zend_class_entry *class_type TSRMLS_DC);

void free_php_service(void *object TSRMLS_DC);
void free_php_service_runnable(void *object TSRMLS_DC);

PHP_METHOD(Service, __construct);
PHP_METHOD(Service, action);
PHP_METHOD(ServiceRunnable, __construct);

typedef struct _php_service {
    zend_object std;

    Service service;
} php_service;

typedef struct _php_service_runnable {
    zend_object std;

    ServiceRunnable service_runnable;
} php_service_runnable;

#endif
