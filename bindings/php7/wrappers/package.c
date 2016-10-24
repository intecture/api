/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "package.h"
#include "host.h"
#include "command.h"
#include <zend_exceptions.h>

extern zend_class_entry *inapi_ce_host, *inapi_ce_package_ex;

static inline php_package * php_package_fetch_object(zend_object *obj) {
      return (php_package *)((char *)obj - XtOffsetOf(php_package, std));
}

#define Z_PKG_OBJ_P(zv) php_package_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(Package, __construct) {
    zval *phost;
    php_host *host;
    char *name;
    size_t name_len;
    long providers = 0;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs|l", &phost, &name, &name_len, &providers) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_package_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    Package *package = package_new(host->host, name, providers);

    if (!package) {
        zend_throw_exception(inapi_ce_package_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    php_package *intern = Z_PKG_OBJ_P(getThis());
    intern->package = package;
}

PHP_METHOD(Package, is_installed) {
    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    php_package *intern = Z_PKG_OBJ_P(getThis());
    bool *installed = package_is_installed(intern->package);

    if (!installed) {
        zend_throw_exception(inapi_ce_package_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    if (*installed == true) {
        RETURN_TRUE;
    } else {
        RETURN_FALSE;
    }
}

PHP_METHOD(Package, install) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_package_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_package *intern = Z_PKG_OBJ_P(getThis());
    CommandResult *result = package_install(intern->package, host->host);

    if (result) {
        array_init(return_value);
        add_assoc_long(return_value, "exit_code", result->exit_code);
        add_assoc_string(return_value, "stdout", result->stdout);
        add_assoc_string(return_value, "stderr", result->stderr);
    } else {
        RETURN_NULL();
    }
}

PHP_METHOD(Package, uninstall) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_package_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_package *intern = Z_PKG_OBJ_P(getThis());
    CommandResult *result = package_install(intern->package, host->host);

    if (result) {
        array_init(return_value);
        add_assoc_long(return_value, "exit_code", result->exit_code);
        add_assoc_string(return_value, "stdout", result->stdout);
        add_assoc_string(return_value, "stderr", result->stderr);
    } else {
        RETURN_NULL();
    }
}
