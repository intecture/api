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
 * Host Class
 */

zend_class_entry *inapi_ce_host;

static zend_function_entry host_methods[] = {
  PHP_ME(Host, __construct, NULL, ZEND_ACC_PRIVATE|ZEND_ACC_CTOR)
  PHP_ME(Host, connect, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
  PHP_ME(Host, connect_endpoint, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
  PHP_ME(Host, connect_payload, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
  PHP_ME(Host, data, NULL, ZEND_ACC_PUBLIC)
  {NULL, NULL, NULL}
};

void inapi_init_host(TSRMLS_D) {
  zend_class_entry ce;

  INIT_CLASS_ENTRY(ce, "Intecture\\Host", host_methods);
  ce.create_object = create_php_host;
  inapi_ce_host = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_host(zend_class_entry *class_type TSRMLS_DC) {
  zend_object_value retval;
  php_host  *intern;
  zval *tmp;

  intern = (php_host*)emalloc(sizeof(php_host));
  memset(intern, 0, sizeof(php_host));
  ALLOC_INIT_ZVAL(intern->data);

  zend_object_std_init(&intern->std, class_type TSRMLS_CC);
  object_properties_init(&intern->std, class_type);

  retval.handle = zend_objects_store_put(
      intern,
      (zend_objects_store_dtor_t) zend_objects_destroy_object,
      free_php_host,
      NULL TSRMLS_CC
  );
  retval.handlers = zend_get_std_object_handlers();

  return retval;
}

void free_php_host(void *object TSRMLS_DC) {
  php_host *host = (php_host*)object;
  host_close(host->host);
  zval_dtor(host->data);
  efree(host);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_host_exception;

void inapi_init_host_exception(TSRMLS_D) {
  zend_class_entry e;

  INIT_CLASS_ENTRY(e, "Intecture\\HostException", NULL);
  inapi_ce_host_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Host Methods
 */

PHP_METHOD(Host, __construct) {
}

PHP_METHOD(Host, connect) {
    php_host *intern;
    char *path;
    int path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &path, &path_len) == FAILURE) {
        return;
    }

    Host *host = host_connect(path);

    if (!host) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    void *data = host_data(host);

    if (!data) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    object_init_ex(return_value, inapi_ce_host);
    intern = (php_host*)zend_object_store_get_object(return_value TSRMLS_CC);
    intern->host = host;
    unwrap_value(data, 7, intern->data TSRMLS_CC); // 7 = Object
}

PHP_METHOD(Host, connect_endpoint) {
    php_host *intern;
    char *hostname;
    int hostname_len;
    long api_port, upload_port;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sll", &hostname, &hostname_len, &api_port, &upload_port) == FAILURE) {
        return;
    }

    Host *host = host_connect_endpoint(hostname, api_port, upload_port);

    if (!host) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    void *data = host_data(host);

    if (!data) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    object_init_ex(return_value, inapi_ce_host);
    intern = (php_host*)zend_object_store_get_object(return_value TSRMLS_CC);
    intern->host = host;
    unwrap_value(data, 7, intern->data TSRMLS_CC); // 7 = Object
}

PHP_METHOD(Host, connect_payload) {
    php_host *intern;
    char *api_endpoint, *file_endpoint;
    int api_len, file_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "ss", &api_endpoint, &api_len, &file_endpoint, &file_len) == FAILURE) {
        return;
    }

    Host *host = host_connect_payload(api_endpoint, file_endpoint);

    if (!host) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    void *data = host_data(host);

    if (!data) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    object_init_ex(return_value, inapi_ce_host);
    intern = (php_host*)zend_object_store_get_object(return_value TSRMLS_CC);
    intern->host = host;
    unwrap_value(data, 7, intern->data TSRMLS_CC); // 7 = Object
}

PHP_METHOD(Host, data) {
    php_host *intern;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    intern = (php_host*)zend_object_store_get_object(getThis() TSRMLS_CC);
    RETURN_ZVAL(intern->data, true, false);
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
            RETURN_STRING((char *)value, 1);
            break;

        // Array
        case 6:
            array_init(return_value);

            ValueArray *a = value;

            int i = 0;
            for (i = 0; i < a->length; i++) {
                int retval = get_value_type(a->ptr[i], NULL);

                if (retval < 0) {
                    zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
                    return;
                }

                enum DataType dtype = (enum DataType)retval;

                void *v = get_value(a->ptr[i], dtype, NULL);

                if (!v) {
                    add_next_index_null(return_value);
                } else {
                    ALLOC_INIT_ZVAL(val);
                    unwrap_value(v, dtype, val TSRMLS_CC);
                    add_next_index_zval(return_value, val);
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

                // Only attempt to get value if it isn't null (DataType[0])
                if (dtype > 0) {
                    void *v = get_value(value, dtype, json_p);
                    assert(v);

                    ALLOC_INIT_ZVAL(val);
                    unwrap_value(v, dtype, val TSRMLS_CC);
                    add_assoc_zval(return_value, k->ptr[i], val);
                } else {
                    add_assoc_null(return_value, k->ptr[i]);
                }
            }
            break;
    }
}

int get_check_host(zval *phost, php_host **host TSRMLS_DC) {
    switch (Z_TYPE_P(phost)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(phost), inapi_ce_host TSRMLS_CC)) {
                return 1;
            }
            break;

        default:
            return 1;
            break;
    }

    *host = (php_host *)zend_object_store_get_object(phost TSRMLS_CC);

    return 0;
}
