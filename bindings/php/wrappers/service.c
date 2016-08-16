/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "service.h"
#include "host.h"
#include "command.h"
#include <string.h>
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
 * Service Class
 */

zend_class_entry *inapi_ce_service;

static zend_function_entry service_methods[] = {
    PHP_ME(Service, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Service, action, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_service(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\Service", service_methods);
    ce.create_object = create_php_service;
    inapi_ce_service = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_service(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_service *intern;
    zval *tmp;

    intern = (php_service*)emalloc(sizeof(php_service));
    memset(intern, 0, sizeof(php_service));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_service,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_service(void *object TSRMLS_DC) {
    php_service *service = (php_service*)object;
    efree(service);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_service_exception;

void inapi_init_service_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\ServiceException", NULL);
    inapi_ce_service_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * ServiceRunnable Class
 */

zend_class_entry *inapi_ce_service_runnable;

static zend_function_entry service_runnable_methods[] = {
    PHP_ME(ServiceRunnable, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    {NULL, NULL, NULL}
};

void inapi_init_service_runnable(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\ServiceRunnable", service_runnable_methods);
    ce.create_object = create_php_service_runnable;
    inapi_ce_service_runnable = zend_register_internal_class(&ce TSRMLS_CC);

    zend_declare_class_constant_long(inapi_ce_service_runnable, "COMMAND", 7, RUNNABLE_COMMAND TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_service_runnable, "SERVICE", 7, RUNNABLE_SERVICE TSRMLS_CC);
}

zend_object_value create_php_service_runnable(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_service_runnable *intern;
    zval *tmp;

    intern = (php_service_runnable*)emalloc(sizeof(php_service_runnable));
    memset(intern, 0, sizeof(php_service_runnable));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_service_runnable,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_service_runnable(void *object TSRMLS_DC) {
    php_service_runnable *runnable = (php_service_runnable*)object;

    if (runnable->service_runnable.command != NULL) {
        efree(runnable->service_runnable.command);
    }
    if (runnable->service_runnable.service != NULL) {
        efree(runnable->service_runnable.service);
    }

    efree(runnable);
}

/*
 * Service Methods
 */

PHP_METHOD(Service, __construct) {
    php_service *intern;
    zval *pactions;
    zval *pmapped_actions = NULL;
    ServiceMappedAction *mapped_actions = NULL;
    int mapped_count = 0;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &pactions, &pmapped_actions) == FAILURE) {
        return;
    }

    if (pmapped_actions != NULL) {
        zval **mapped_action;
        HashTable *mapped_hash;
        HashPosition mapped_ptr;
        int mapped_len;

        mapped_hash = Z_ARRVAL_P(pmapped_actions);
        mapped_len = zend_hash_num_elements(mapped_hash) * sizeof(ServiceMappedAction);
        mapped_actions = malloc(mapped_len);
        memset(mapped_actions, 0, mapped_len);

        for (zend_hash_internal_pointer_reset_ex(mapped_hash, &mapped_ptr);
             zend_hash_get_current_data_ex(mapped_hash, (void**) &mapped_action, &mapped_ptr) == SUCCESS;
             zend_hash_move_forward_ex(mapped_hash, &mapped_ptr)) {
            char *key;
            unsigned int key_len;
            unsigned long index;
            zend_hash_get_current_key_ex(mapped_hash, &key, &key_len, &index, 0, &mapped_ptr);

            ServiceMappedAction ma = { .action = key, .mapped_action = Z_STRVAL_PP(mapped_action) };
            mapped_actions[mapped_count] = ma;
            mapped_count++;
        }
    }

    intern = (php_service*)zend_object_store_get_object(getThis() TSRMLS_CC);

    switch (Z_TYPE_P(pactions)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(pactions), inapi_ce_service_runnable TSRMLS_CC)) {
                zend_throw_exception(inapi_ce_service_exception, "The first argument must be an instance of Intecture\\ServiceRunnable", 1001 TSRMLS_CC);
                return;
            }

            php_service_runnable *runnable = (php_service_runnable*)zend_object_store_get_object(pactions TSRMLS_CC);
            intern->service = service_new_service(runnable->service_runnable, mapped_actions, mapped_count);
            break;

        case IS_ARRAY: ;
            zval **zrunnable;
            HashTable *actions_hash;
            HashPosition actions_ptr;
            int actions_count = 0;
            int actions_len;

            actions_hash = Z_ARRVAL_P(pactions);
            actions_len = zend_hash_num_elements(actions_hash) * sizeof(ServiceAction);
            ServiceAction *actions = malloc(actions_len);
            memset(actions, 0, actions_len);

            for (zend_hash_internal_pointer_reset_ex(actions_hash, &actions_ptr);
                 zend_hash_get_current_data_ex(actions_hash, (void**) &zrunnable, &actions_ptr) == SUCCESS;
                 zend_hash_move_forward_ex(actions_hash, &actions_ptr)) {
                if (Z_TYPE_P(*zrunnable) != IS_OBJECT || !instanceof_function(Z_OBJCE_P(*zrunnable), inapi_ce_service_runnable TSRMLS_CC)) {
                    zend_throw_exception(inapi_ce_service_exception, "Array values must be instances of Intecture\\ServiceRunnable", 1001 TSRMLS_CC);
                    return;
                }

                php_service_runnable *runnable = (php_service_runnable *)zend_object_store_get_object(*zrunnable TSRMLS_CC);

                char *key;
                unsigned int key_len;
                unsigned long index;
                zend_hash_get_current_key_ex(actions_hash, &key, &key_len, &index, 0, &actions_ptr);

                ServiceAction a = { .action = key, .runnable = runnable->service_runnable };
                actions[actions_count] = a;
                actions_count++;
            }

            intern->service = service_new_map(actions, actions_count, mapped_actions, mapped_count);
            break;

        default:
            zend_throw_exception(inapi_ce_service_exception, "The first argument must be an instance of Intecture\\ServiceRunnable", 1001 TSRMLS_CC);
    }
}

PHP_METHOD(Service, action) {
    php_service *intern;
    zval *phost;
    php_host *host;
    char *action;
    int action_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &action, &action_len) == FAILURE) {
        return;
    }

    intern = (php_service*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_service_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    CommandResult *result = service_action(&intern->service, &host->host, action);

    if (result != NULL) {
        array_init(return_value);
        add_assoc_long(return_value, "exit_code", result->exit_code);
        add_assoc_string(return_value, "stdout", result->stdout, 1);
        add_assoc_string(return_value, "stderr", result->stderr, 1);
    } else {
        RETURN_NULL();
    }
}

/*
 * ServiceRunnable Methods
 */

 PHP_METHOD(ServiceRunnable, __construct) {
    php_service_runnable *intern;
    char *runnable;
    int runnable_len;
    long type;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sl", &runnable, &runnable_len, &type) == FAILURE) {
        return;
    }

    intern = (php_service_runnable*)zend_object_store_get_object(getThis() TSRMLS_CC);

    switch (type) {
        case RUNNABLE_COMMAND: ;
            intern->service_runnable.command = (char *) emalloc(runnable_len+1);
            memset(intern->service_runnable.command, 0, runnable_len+1);
            strncpy(intern->service_runnable.command, runnable, runnable_len);
            break;
        case RUNNABLE_SERVICE: ;
            intern->service_runnable.service = (char *) emalloc(runnable_len+1);
            memset(intern->service_runnable.service, 0, runnable_len+1);
            strncpy(intern->service_runnable.service, runnable, runnable_len);
            break;
        default:
            zend_throw_exception(inapi_ce_service_exception, "Invalid Runnable type. Must be RUNNABLE_COMMAND or RUNNABLE_SERVICE.", 1002 TSRMLS_CC);
    }
 }
