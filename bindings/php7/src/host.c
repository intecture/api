/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "host.h"
#include <zend_exceptions.h>

extern zend_class_entry *inapi_ce_host, *inapi_ce_host_ex;

static inline php_host * php_host_fetch_object(zend_object *obj) {
      return (php_host *)((char *)obj - XtOffsetOf(php_host, std));
}

#define Z_HOST_OBJ_P(zv) php_host_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(Host, __construct) {
}

PHP_METHOD(Host, connect) {
    char *path;
    size_t path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &path, &path_len) == FAILURE) {
        return;
    }

    Host *host = host_connect(path);

    if (!host) {
        zend_throw_exception(inapi_ce_host_ex, geterr(), 1000);
        return;
    }

    void *data = host_data(host);

    if (!data) {
        zend_throw_exception(inapi_ce_host_ex, geterr(), 1000);
        return;
    }

    object_init_ex(return_value, inapi_ce_host);
    php_host *intern = Z_HOST_OBJ_P(return_value);
    intern->host = host;
    unwrap_value(data, 7, &intern->data TSRMLS_CC); // 7 = Object
}

PHP_METHOD(Host, connect_endpoint) {
    char *hostname;
    size_t hostname_len;
    long api_port, upload_port;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sll", &hostname, &hostname_len, &api_port, &upload_port) == FAILURE) {
        return;
    }

    Host *host = host_connect_endpoint(hostname, api_port, upload_port);

    if (!host) {
        zend_throw_exception(inapi_ce_host_ex, geterr(), 1000);
        return;
    }

    void *data = host_data(host);

    if (!data) {
        zend_throw_exception(inapi_ce_host_ex, geterr(), 1000);
        return;
    }

    object_init_ex(return_value, inapi_ce_host);
    php_host *intern = Z_HOST_OBJ_P(return_value);
    intern->host = host;
    unwrap_value(data, 7, &intern->data TSRMLS_CC); // 7 = Object
}

PHP_METHOD(Host, connect_payload) {
    char *api_endpoint, *file_endpoint;
    size_t api_len, file_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "ss", &api_endpoint, &api_len, &file_endpoint, &file_len) == FAILURE) {
        return;
    }

    Host *host = host_connect_payload(api_endpoint, file_endpoint);

    if (!host) {
        zend_throw_exception(inapi_ce_host_ex, geterr(), 1000);
        return;
    }

    void *data = host_data(host);

    if (!data) {
        zend_throw_exception(inapi_ce_host_ex, geterr(), 1000);
        return;
    }

    object_init_ex(return_value, inapi_ce_host);
    php_host *intern = Z_HOST_OBJ_P(return_value);
    intern->host = host;
    unwrap_value(data, 7, &intern->data TSRMLS_CC); // 7 = Object
}

PHP_METHOD(Host, data) {
    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    php_host *intern = Z_HOST_OBJ_P(getThis());
    RETURN_ZVAL(&intern->data, true, false);
}

void unwrap_value(void *value, enum DataType dtype, zval *return_value TSRMLS_DC) {
    zval *retval, *val;

    switch (dtype) {
        // Null
        case 0:
            RETURN_NULL();
            break;

        // Boolean
        case 1:
            if (!!value) {
                RETURN_TRUE;
            } else {
                RETURN_FALSE;
            }
            break;

        // Int64
        case 2:
        // Uint64
        case 3:
            RETURN_LONG(*(long *)value);
            break;

        // Double
        case 4:
            RETURN_DOUBLE(*(double *)value);
            break;

        // String
        case 5:
            RETURN_STRING((char *)value);
            break;

        // Array
        case 6:
            array_init(return_value);

            ValueArray *a = value;

            int i = 0;
            for (i = 0; i < a->length; i++) {
                int retval = get_value_type(a->ptr[i], NULL);

                if (retval < 0) {
                    zend_throw_exception(inapi_ce_host_ex, geterr(), 1000);
                    return;
                }

                enum DataType dtype = (enum DataType)retval;

                void *v = get_value(a->ptr[i], dtype, NULL);

                if (!v) {
                    add_next_index_null(return_value);
                } else {
                    zval val;
                    unwrap_value(v, dtype, &val TSRMLS_CC);
                    add_next_index_zval(return_value, &val);
                }
            }
            break;

        // Object
        case 7:
            array_init(return_value);

            ValueKeysArray *k = get_value_keys(value, NULL);

            i = 0;
            for (i = 0; i < k->length; i++) {
                char json_p[256] = "/";
                strncat(json_p, k->ptr[i], 255);
                int retval = get_value_type(value, json_p);
                assert(retval > -1);
                enum DataType dtype = (enum DataType)retval;
                void *v = get_value(value, dtype, json_p);
                assert(v);

                zval val;
                unwrap_value(v, dtype, &val TSRMLS_CC);
                add_assoc_zval(return_value, k->ptr[i], &val);
            }
            break;
    }
}

php_host *check_host(zval *phost TSRMLS_DC) {
    switch (Z_TYPE_P(phost)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(phost), inapi_ce_host TSRMLS_CC)) {
                return NULL;
            }
            break;

        default:
            return NULL;
            break;
    }

    return Z_HOST_OBJ_P(phost);
}
