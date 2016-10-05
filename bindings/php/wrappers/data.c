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

zend_class_entry *inapi_ce_value;

static zend_function_entry value_methods[] = {
    PHP_ME(Value, get, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_value(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\Value", value_methods);
    ce.create_object = create_php_value;
    inapi_ce_value = zend_register_internal_class(&ce TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_value, "BOOL", 4, 0 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_value, "INT", 3, 1 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_value, "DOUBLE", 6, 3 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_value, "STRING", 6, 4 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_value, "ARR", 3, 5 TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_value, "OBJECT", 6, 6 TSRMLS_CC);
}

zend_object_value create_php_value(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_value  *intern;

    intern = (php_value*)emalloc(sizeof(php_value));
    memset(intern, 0, sizeof(php_value));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_value,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_value(void *object TSRMLS_DC) {
    php_value *value = (php_value*)object;
    free_value(value->value);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_value_exception;

void inapi_init_value_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\ValueException", NULL);
    inapi_ce_value_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

PHP_FUNCTION(data_open) {
    php_value *intern;
    char *path;
    int path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &path, &path_len) == FAILURE) {
        return;
    }

    void *value = data_open(path);

    if (!value) {
        zend_throw_exception(inapi_ce_value_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    object_init_ex(return_value, inapi_ce_value);
    intern = (php_value *)zend_object_store_get_object(return_value TSRMLS_CC);
    intern->value = value;
}

/*
 * Value Methods
 */

PHP_METHOD(Value, get) {
    php_value *intern, *pvalue;
    zval *retval, *obj;
    long dtype;
    char *pointer;
    int pointer_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "l|s", &dtype, &pointer, &pointer_len) == FAILURE) {
        return;
    }

    if (dtype < 0 || dtype > 6) {
        zend_throw_exception(inapi_ce_value_exception, "The first argument must be a Value class constant", 1010 TSRMLS_CC);
        return;
    }

    intern = (php_value*)zend_object_store_get_object(getThis() TSRMLS_CC);

    void *v = get_value(intern->value, dtype, pointer);

    // Try uint if int not found
    if (!v && dtype == 1) {
        v = get_value(intern->value, 2, pointer);
    }

    if (!v) {
        RETURN_NULL();
    } else {
        switch (dtype) {
            // Boolean
            case 0:
                if (!!v) {
                    RETURN_TRUE;
                } else {
                    RETURN_FALSE;
                }
                break;

            // Int64
            case 1:
                RETURN_LONG(*(long *)v);
                break;

            // Double
            case 3:
                RETURN_DOUBLE(*(double *)v);
                break;

            // String
            case 4:
                RETURN_STRING((char *)v, 1);
                break;

            // Array
            case 5:
                array_init(return_value);

                ValueArray *a = v;

                int i = 0;
                for (i = 0; i < a->length; i++) {
                    ALLOC_INIT_ZVAL(obj);
                    object_init_ex(obj, inapi_ce_value);
                    pvalue = (php_value *)zend_object_store_get_object(obj TSRMLS_CC);
                    pvalue->value = &a->ptr[i];
                    add_next_index_zval(return_value, obj);
                }
                break;

            // Object
            case 6:
                object_init_ex(return_value, inapi_ce_value);
                pvalue = (php_value *)zend_object_store_get_object(return_value TSRMLS_CC);
                pvalue->value = v;
                break;
        }
    }
}
