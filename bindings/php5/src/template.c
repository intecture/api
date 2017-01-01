/*
 Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "template.h"
#include "host.h"
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
 * Template Class
 */

zend_class_entry *inapi_ce_template;

static zend_function_entry template_methods[] = {
    PHP_ME(Template, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Template, render, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_template(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\Template", template_methods);
    ce.create_object = create_php_template;
    inapi_ce_template = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_template(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_template *intern;
    zval *tmp;

    intern = (php_template*)emalloc(sizeof(php_template));
    memset(intern, 0, sizeof(php_template));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_template,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_template(void *object TSRMLS_DC) {
    php_template *template = (php_template*)object;
    if (template->template) {
        int rc = template_free(template->template);
        assert(rc == 0);
    }
    efree(template);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_template_exception;

void inapi_init_template_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\TemplateException", NULL);
    inapi_ce_template_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Template Methods
 */

PHP_METHOD(Template, __construct) {
    php_template *intern;
    char *path;
    int path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &path, &path_len) == FAILURE) {
        return;
    }

    intern = (php_template*)zend_object_store_get_object(getThis() TSRMLS_CC);

    Template *template = template_new(path);

    if (!template) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->template = template;
}

PHP_METHOD(Template, render) {
    php_template *intern;
    zval *arr;
    HashTable *arr_hash;
    int fd, rc;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "a", &arr) == FAILURE) {
        return;
    }

    intern = (php_template*)zend_object_store_get_object(getThis() TSRMLS_CC);

    arr_hash = Z_ARRVAL_P(arr);

    if (array_is_hash(arr_hash TSRMLS_CC)) {
        MapBuilder *hbuilder = map_new();
        rc = build_map(arr_hash, hbuilder TSRMLS_CC);

        if (rc != 0) {
            zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
            return;
        }

        fd = template_render_map(intern->template, hbuilder);
    } else {
        VecBuilder *vbuilder = vec_new();
        rc = build_vec(arr_hash, vbuilder TSRMLS_CC);

        if (rc != 0) {
            zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
            return;
        }

        fd = template_render_vec(intern->template, vbuilder);
    }

    if (fd == 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    RETURN_LONG(fd);
}

bool array_is_hash(HashTable *arr_hash TSRMLS_DC) {
    bool is_hash;
    HashPosition pointer;
    zval **data;

    for (zend_hash_internal_pointer_reset_ex(arr_hash, &pointer);
         zend_hash_get_current_data_ex(arr_hash, (void**) &data, &pointer) == SUCCESS;
         zend_hash_move_forward_ex(arr_hash, &pointer)) {

        // String index == 1
        // Integer index == 2
        int type = zend_hash_get_current_key_type_ex(arr_hash, &pointer);

        if (type == 1) {
            return true;
        }
    }

    return false;
}

int build_map(HashTable *arr_hash, MapBuilder *builder TSRMLS_DC) {
    HashPosition pointer;
    HashTable *arr_hash1;
    int rc;
    zval **data;

    for (zend_hash_internal_pointer_reset_ex(arr_hash, &pointer);
         zend_hash_get_current_data_ex(arr_hash, (void**) &data, &pointer) == SUCCESS;
         zend_hash_move_forward_ex(arr_hash, &pointer)) {

        char *key, intbuf[256], indexkey[256];
        unsigned int key_len;
        unsigned long index;

        // Convert integer indexes to char
        if (zend_hash_get_current_key_ex(arr_hash, &key, &key_len, &index, 0, &pointer) == HASH_KEY_IS_LONG) {
            sprintf(indexkey, "%lu", index);
            key = &indexkey[0];
        }

        if (Z_TYPE_PP(data) == IS_BOOL) {
            zend_bool b = Z_BVAL_PP(data);
            rc = map_insert_bool(builder, key, b);
        }
        else if (Z_TYPE_PP(data) == IS_DOUBLE) {
            sprintf(intbuf, "%g", Z_DVAL_PP(data));
            rc = map_insert_str(builder, key, &intbuf[0]);
        }
        else if (Z_TYPE_PP(data) == IS_LONG) {
            sprintf(intbuf, "%li", Z_LVAL_PP(data));
            rc = map_insert_str(builder, key, &intbuf[0]);
        }
        else if (Z_TYPE_PP(data) == IS_STRING) {
            rc = map_insert_str(builder, key, Z_STRVAL_PP(data));
        }
        else if (Z_TYPE_PP(data) == IS_ARRAY) {
            arr_hash1 = Z_ARRVAL_PP(data);

            if (array_is_hash(arr_hash1 TSRMLS_CC)) {
                MapBuilder *hbuilder = map_new();
                rc = build_map(arr_hash1, hbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = map_insert_map(builder, key, hbuilder);
                }
            } else {
                VecBuilder *vbuilder = vec_new();
                rc = build_vec(arr_hash1, vbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = map_insert_vec(builder, key, vbuilder);
                }
            }
        }
        else if (Z_TYPE_PP(data) == IS_NULL) {
            rc = map_insert_bool(builder, key, false);
        } else {
            zend_throw_exception(inapi_ce_template_exception, "Array value cannot be a resource or object", 1001 TSRMLS_CC);
            rc = -1;
        }

        if (rc != 0) {
            return rc;
        }
    }

    return rc;
}

int build_vec(HashTable *arr_hash, VecBuilder *builder TSRMLS_DC) {
    HashPosition pointer;
    HashTable *arr_hash1;
    int rc;
    zval **data;

    for (zend_hash_internal_pointer_reset_ex(arr_hash, &pointer);
         zend_hash_get_current_data_ex(arr_hash, (void**) &data, &pointer) == SUCCESS;
         zend_hash_move_forward_ex(arr_hash, &pointer)) {

        char intbuf[256];

        if (Z_TYPE_PP(data) == IS_BOOL) {
            zend_bool b = Z_BVAL_PP(data);
            rc = vec_push_bool(builder, b);
        }
        else if (Z_TYPE_PP(data) == IS_DOUBLE) {
            sprintf(intbuf, "%g", Z_DVAL_PP(data));
            rc = vec_push_str(builder, &intbuf[0]);
        }
        else if (Z_TYPE_PP(data) == IS_LONG) {
            sprintf(intbuf, "%li", Z_LVAL_PP(data));
            rc = vec_push_str(builder, &intbuf[0]);
        }
        else if (Z_TYPE_PP(data) == IS_STRING) {
            rc = vec_push_str(builder, Z_STRVAL_PP(data));
        }
        else if (Z_TYPE_PP(data) == IS_ARRAY) {
            arr_hash1 = Z_ARRVAL_PP(data);

            if (array_is_hash(arr_hash1 TSRMLS_CC)) {
                MapBuilder *hbuilder = map_new();
                rc = build_map(arr_hash1, hbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = vec_push_map(builder, hbuilder);
                }
            } else {
                VecBuilder *vbuilder = vec_new();
                rc = build_vec(arr_hash1, vbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = vec_push_vec(builder, vbuilder);
                }
            }
        }
        else if (Z_TYPE_PP(data) == IS_NULL) {
            rc = vec_push_bool(builder, false);
        } else {
            zend_throw_exception(inapi_ce_template_exception, "Array value cannot be a resource or object", 1001 TSRMLS_CC);
            rc = -1;
        }

        if (rc != 0) {
            return rc;
        }
    }

    return rc;
}
