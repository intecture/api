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

/* PHP 5.4 */
#if PHP_VERSION_ID < 50399
# define object_properties_init(zo, class_type) { \
    zval *tmp; \
    zend_hash_copy((*zo).properties, \
        &class_type->default_properties, \
        (copy_ctor_func_t) zval_add_ref, \
        (void *) &tmp, \
        sizeof(zval *)); \
    }
#endif

/*
 * Package Class
 */

zend_class_entry *inapi_ce_package;

static zend_function_entry package_methods[] = {
    PHP_ME(Package, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Package, is_installed, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Package, install, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Package, uninstall, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_package(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\Package", package_methods);
    ce.create_object = create_php_package;
    inapi_ce_package = zend_register_internal_class(&ce TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_package, "PROVIDER_APT", 12, 1 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_package, "PROVIDER_DNF", 12, 2 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_package, "PROVIDER_HOMEBREW", 17, 3 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_package, "PROVIDER_MACPORTS", 17, 4 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_package, "PROVIDER_PKG", 12, 5 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_package, "PROVIDER_PORTS", 14, 6 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_package, "PROVIDER_YUM", 12, 7 TSRMLS_CC);
}

zend_object_value create_php_package(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_package  *intern;
    zval *tmp;

    intern = (php_package*)emalloc(sizeof(php_package));
    memset(intern, 0, sizeof(php_package));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_package,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_package(void *object TSRMLS_DC) {
    php_package *package = (php_package*)object;
    efree(package);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_package_exception;

void inapi_init_package_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\PackageException", NULL);
    inapi_ce_package_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Package Methods
 */

PHP_METHOD(Package, __construct) {
    php_package *intern;
    zval *phost;
    php_host *host;
    char *name;
    int name_len;
    long providers = 0;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs|l", &phost, &name, &name_len, &providers) == FAILURE) {
        return;
    }

    intern = (php_package*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_package_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    Package *package = package_new(host->host, name, providers);

    if (!package) {
        zend_throw_exception(inapi_ce_package_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->package = package;
}

PHP_METHOD(Package, is_installed) {
    php_package *intern;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    intern = (php_package*)zend_object_store_get_object(getThis() TSRMLS_CC);

    bool *installed = package_is_installed(intern->package);

    if (!installed) {
        zend_throw_exception(inapi_ce_package_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    if (*installed == true) {
        RETURN_TRUE;
    } else {
        RETURN_FALSE;
    }
}

PHP_METHOD(Package, install) {
    php_package *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_package*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_package_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    CommandResult cmd_result;
    enum PackageResult *result = package_install(intern->package, host->host, &cmd_result);

    if (!result) {
        zend_throw_exception(inapi_ce_package_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    if (*result == 0) {
        array_init(return_value);
        add_assoc_long(return_value, "exit_code", cmd_result.exit_code);
        add_assoc_string(return_value, "stdout", cmd_result.stdout, 1);
        add_assoc_string(return_value, "stderr", cmd_result.stderr, 1);
    } else {
        RETURN_NULL();
    }
}

PHP_METHOD(Package, uninstall) {
    php_package *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_package*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_package_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    CommandResult cmd_result;
    enum PackageResult *result = package_uninstall(intern->package, host->host, &cmd_result);

    if (!*result) {
        zend_throw_exception(inapi_ce_package_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    if (result == 0) {
        array_init(return_value);
        add_assoc_long(return_value, "exit_code", cmd_result.exit_code);
        add_assoc_string(return_value, "stdout", cmd_result.stdout, 1);
        add_assoc_string(return_value, "stderr", cmd_result.stderr, 1);
    } else {
        RETURN_NULL();
    }
}
