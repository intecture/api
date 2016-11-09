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

extern zend_class_entry *inapi_ce_host, *inapi_ce_service_ex, *inapi_ce_service_runnable;

static inline php_service * php_service_fetch_object(zend_object *obj) {
      return (php_service *)((char *)obj - XtOffsetOf(php_service, std));
}

#define Z_SVC_OBJ_P(zv) php_service_fetch_object(Z_OBJ_P(zv));

static inline php_service_runnable * php_service_runnable_fetch_object(zend_object *obj) {
      return (php_service_runnable *)((char *)obj - XtOffsetOf(php_service_runnable, std));
}

#define Z_SVC_RUN_OBJ_P(zv) php_service_runnable_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(Service, __construct) {
    Service *service;
    zval *zactions, *zv;
    zval *zmapped_actions = NULL;
    zend_string *zk;
    ServiceMappedAction *mapped_actions = NULL;
    int mapped_count = 0;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &zactions, &zmapped_actions) == FAILURE) {
        return;
    }

    if (zmapped_actions) {
        HashTable *ht = Z_ARRVAL_P(zmapped_actions);
        int len = zend_hash_num_elements(ht) * sizeof(ServiceMappedAction);
        // Don't free this ptr as Vec::from_raw_parts claims ownership
        // when passed to Rust.
        mapped_actions = malloc(len);
        memset(mapped_actions, 0, len);

        ZEND_HASH_FOREACH_STR_KEY_VAL(ht, zk, zv) {
            if (zk) {
                ServiceMappedAction ma = { .action = ZSTR_VAL(zk), .mapped_action = Z_STRVAL_P(zv) };
                mapped_actions[mapped_count] = ma;
                mapped_count++;
            }
        } ZEND_HASH_FOREACH_END();
    }

    php_service *intern = Z_SVC_OBJ_P(getThis());

    switch (Z_TYPE_P(zactions)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(zactions), inapi_ce_service_runnable TSRMLS_CC)) {
                zend_throw_exception(inapi_ce_service_ex, "The first argument must be an instance of Intecture\\ServiceRunnable", 1001 TSRMLS_CC);
                return;
            }

            php_service_runnable *runnable = Z_SVC_RUN_OBJ_P(zactions);
            service = service_new_service(runnable->service_runnable, mapped_actions, mapped_count);

            if (!service) {
                zend_throw_exception(inapi_ce_service_ex, geterr(), 1000 TSRMLS_CC);
                return;
            }

            intern->service = service;
            break;

        case IS_ARRAY: ;
            HashTable *ht = Z_ARRVAL_P(zactions);
            int len = zend_hash_num_elements(ht) * sizeof(ServiceAction);
            // Don't free this ptr as Vec::from_raw_parts claims ownership
            // when passed to Rust.
            ServiceAction *actions = malloc(len);
            memset(actions, 0, len);
            int actions_count = 0;

            ZEND_HASH_FOREACH_STR_KEY_VAL(ht, zk, zv) {
                if (zk) {
                    if (Z_TYPE_P(zv) != IS_OBJECT || !instanceof_function(Z_OBJCE_P(zv), inapi_ce_service_runnable TSRMLS_CC)) {
                        zend_throw_exception(inapi_ce_service_ex, "Array values must be instances of Intecture\\ServiceRunnable", 1001 TSRMLS_CC);
                        return;
                    }

                    php_service_runnable *runnable = Z_SVC_RUN_OBJ_P(zv);
                    ServiceAction a = { .action = ZSTR_VAL(zk), .runnable = runnable->service_runnable };
                    actions[actions_count] = a;
                    actions_count++;
                }
            } ZEND_HASH_FOREACH_END();

            service = service_new_map(actions, actions_count, mapped_actions, mapped_count);

            if (!service) {
                zend_throw_exception(inapi_ce_service_ex, geterr(), 1000 TSRMLS_CC);
                return;
            }

            intern->service = service;
            break;

        default:
            zend_throw_exception(inapi_ce_service_ex, "The first argument must be an instance of Intecture\\ServiceRunnable", 1001 TSRMLS_CC);
    }
}

PHP_METHOD(Service, action) {
    zval *phost;
    php_host *host;
    char *action;
    size_t action_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &action, &action_len) == FAILURE) {
        return;
    }

    host = check_host(phost TSRMLS_CC);
    if (!host) {
        zend_throw_exception(inapi_ce_service_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_service *intern = Z_SVC_OBJ_P(getThis());

    CommandResult *result = service_action(intern->service, host->host, action);

    if (result != NULL) {
        array_init(return_value);
        add_assoc_long(return_value, "exit_code", result->exit_code);
        add_assoc_string(return_value, "stdout", result->stdout);
        add_assoc_string(return_value, "stderr", result->stderr);

        int rc = command_result_free(result);
        if (rc != 0) {
            zend_throw_exception(inapi_ce_service_ex, "Could not free internal CommandResult struct", 1001 TSRMLS_CC);
            return;
        }
    } else {
        RETURN_NULL();
    }
}

/*
 * ServiceRunnable Methods
 */

 PHP_METHOD(ServiceRunnable, __construct) {
    char *runnable;
    size_t runnable_len;
    long type;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sl", &runnable, &runnable_len, &type) == FAILURE) {
        return;
    }

    php_service_runnable *intern = Z_SVC_RUN_OBJ_P(getThis());

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
            zend_throw_exception(inapi_ce_service_ex, "Invalid Runnable type. Must be RUNNABLE_COMMAND or RUNNABLE_SERVICE.", 1002 TSRMLS_CC);
    }
 }
