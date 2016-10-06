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
 * Value Class
 */

zend_class_entry *inapi_ce_data;

static zend_function_entry data_methods[] = {
    PHP_ME(Data, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Data, get, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_data(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\Data", data_methods);
    ce.create_object = create_php_data;
    inapi_ce_data = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_data(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_data  *intern;

    intern = (php_data*)emalloc(sizeof(php_data));
    memset(intern, 0, sizeof(php_data));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_data,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_data(void *object TSRMLS_DC) {
    php_data *data = (php_data*)object;
    free_value(data->value);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_data_exception;

void inapi_init_data_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\DataException", NULL);
    inapi_ce_data_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Data Methods
 */

 PHP_METHOD(Data, __construct) {
     php_data *intern;
     char *path;
     int path_len;

     if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &path, &path_len) == FAILURE) {
         return;
     }

     intern = (php_data*)zend_object_store_get_object(getThis() TSRMLS_CC);

     void *value = data_open(path);

     if (!value) {
         zend_throw_exception(inapi_ce_data_exception, geterr(), 1000 TSRMLS_CC);
         return;
     }

     intern->value = value;
 }

PHP_METHOD(Data, get) {
    php_data *intern;
    char *pointer;
    int pointer_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &pointer, &pointer_len) == FAILURE) {
        return;
    }

    intern = (php_data*)zend_object_store_get_object(getThis() TSRMLS_CC);

    enum DataType *dtype = get_value_type(intern->value, pointer);

    if (!dtype) {
        zend_throw_exception(inapi_ce_data_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    void *v = get_value(intern->value, *dtype, pointer);

    if (!v) {
        RETURN_NULL();
    } else {
        return_type(v, *dtype, return_value TSRMLS_CC);
    }
}

void return_type(void *value, enum DataType dtype, zval *return_value TSRMLS_DC) {
    php_data *pdata;
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
                    return_type(v, *dtype, val TSRMLS_CC);
                    add_next_index_zval(return_value, val);
                }
            }
            break;

        // Object
        case 7:
            object_init_ex(return_value, inapi_ce_data);
            pdata = (php_data *)zend_object_store_get_object(return_value TSRMLS_CC);
            pdata->value = value;
            break;
    }
}
