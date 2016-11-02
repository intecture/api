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
#include "php_inapi.h"
#include <zend_exceptions.h>

extern zend_class_entry *inapi_ce_host, *inapi_ce_payload_ex;

static inline php_payload * php_payload_fetch_object(zend_object *obj) {
      return (php_payload *)((char *)obj - XtOffsetOf(php_payload, std));
}

#define Z_PAYLOAD_OBJ_P(zv) php_payload_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(Payload, __construct) {
    char *payload_artifact;
    size_t payload_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &payload_artifact, &payload_len) == FAILURE) {
        return;
    }

    Payload *payload = payload_new(payload_artifact);

    if (!payload) {
        zend_throw_exception(inapi_ce_payload_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    php_payload *intern = Z_PAYLOAD_OBJ_P(getThis());
    intern->payload = payload;
}

PHP_METHOD(Payload, build) {
    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    php_payload *intern = Z_PAYLOAD_OBJ_P(getThis());

    int rc = payload_build(intern->payload);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_payload_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Payload, run) {
    zval *zhost, *zuser_args, *zv;
    php_host *host;
    const char **user_args = NULL;
    int args_len = 0;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &zhost, &zuser_args) == FAILURE) {
        return;
    }

    host = check_host(zhost TSRMLS_CC);
    if (!host) {
        zend_throw_exception(inapi_ce_payload_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    if (zuser_args) {
        HashTable *ht = Z_ARRVAL_P(zuser_args);
        int len = zend_hash_num_elements(ht) * sizeof(char *);
        user_args = malloc(len);
        memset(user_args, 0, len);

        ZEND_HASH_FOREACH_VAL(ht, zv) {
            user_args[args_len] = Z_STRVAL_P(zv);
            args_len++;
        } ZEND_HASH_FOREACH_END();
    }

    php_payload *intern = Z_PAYLOAD_OBJ_P(getThis());
    int rc = payload_run(intern->payload, host->host, user_args, args_len);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_payload_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}
