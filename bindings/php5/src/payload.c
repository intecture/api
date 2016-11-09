/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "payload.h"
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
 * Payload Class
 */

zend_class_entry *inapi_ce_payload;

static zend_function_entry payload_methods[] = {
    PHP_ME(Payload, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Payload, build, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Payload, run, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_payload(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\Payload", payload_methods);
    ce.create_object = create_php_payload;
    inapi_ce_payload = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_payload(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_payload *intern;

    intern = (php_payload*)emalloc(sizeof(php_payload));
    memset(intern, 0, sizeof(php_payload));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_payload,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_payload(void *object TSRMLS_DC) {
    php_payload *payload = (php_payload*)object;
    if (payload->payload) {
        int rc = payload_free(payload->payload);
        assert(rc == 0);
    }
    efree(payload);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_payload_exception;

void inapi_init_payload_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\PayloadException", NULL);
    inapi_ce_payload_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Payload Methods
 */

PHP_METHOD(Payload, __construct) {
    php_payload *intern;
    char *payload_artifact;
    int payload_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &payload_artifact, &payload_len) == FAILURE) {
        return;
    }

    Payload *payload = payload_new(payload_artifact);

    if (!payload) {
        zend_throw_exception(inapi_ce_payload_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern = (php_payload*)zend_object_store_get_object(getThis() TSRMLS_CC);
    intern->payload = payload;
}

PHP_METHOD(Payload, build) {
    php_payload *intern;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    intern = (php_payload*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rc = payload_build(intern->payload);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_payload_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Payload, run) {
    php_payload *intern;
    zval *zhost;
    php_host *host;
    zval *zuser_args = NULL;
    const char **user_args = NULL;
    int args_len = 0;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &zhost, &zuser_args) == FAILURE) {
        return;
    }

    intern = (php_payload*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(zhost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_payload_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    if (zuser_args != NULL) {
        zval **arg;
        HashPosition ptr;

        HashTable *hash = Z_ARRVAL_P(zuser_args);
        int len = zend_hash_num_elements(hash) * sizeof(char *);
        user_args = malloc(len);
        memset(user_args, 0, len);

        for (zend_hash_internal_pointer_reset_ex(hash, &ptr);
             zend_hash_get_current_data_ex(hash, (void**) &arg, &ptr) == SUCCESS;
             zend_hash_move_forward_ex(hash, &ptr)) {
            user_args[args_len] = Z_STRVAL_PP(arg);
            args_len++;
        }
    }

    int rc = payload_run(intern->payload, host->host, user_args, args_len);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_payload_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}
