/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT directory at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This directory may not be copied,
 modified, or distributed except according to those terms.
*/

#include "data.h"
#include "host.h"
#include <zend_exceptions.h>
#include <string.h>

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
 * Exception Class
 */

zend_class_entry *inapi_ce_data_exception;

void inapi_init_data_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\DataException", NULL);
    inapi_ce_data_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

PHP_FUNCTION(data_open) {
    char *path;
    int path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &path, &path_len) == FAILURE) {
        return;
    }

    void *value = data_open(path);

    if (!value) {
        zend_throw_exception(inapi_ce_data_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    unwrap_value(value, 7, return_value TSRMLS_CC); // 7 = Object
    free_value(value);
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
                enum DataType *dtype = get_value_type(a->ptr[i], NULL);

                if (!dtype) {
                    zend_throw_exception(inapi_ce_data_exception, geterr(), 1000 TSRMLS_CC);
                    return;
                }

                void *v = get_value(a->ptr[i], *dtype, NULL);

                if (!v) {
                    add_next_index_null(return_value);
                } else {
                    ALLOC_INIT_ZVAL(val);
                    unwrap_value(v, *dtype, val TSRMLS_CC);
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
                enum DataType *dtype = get_value_type(value, json_p);
                assert(dtype);
                void *v = get_value(value, *dtype, json_p);
                assert(v);

                ALLOC_INIT_ZVAL(val);
                unwrap_value(v, *dtype, val TSRMLS_CC);
                add_assoc_zval(return_value, k->ptr[i], val);
            }
            break;
    }
}
