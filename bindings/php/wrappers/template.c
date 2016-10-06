/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
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
 * MapBuilder Class
 */

zend_class_entry *inapi_ce_mapbuilder;

static zend_function_entry mapbuilder_methods[] = {
    PHP_ME(MapBuilder, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(MapBuilder, insert_str, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(MapBuilder, insert_bool, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(MapBuilder, insert_vec, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(MapBuilder, insert_map, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_mapbuilder(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\MapBuilder", mapbuilder_methods);
    ce.create_object = create_php_mapbuilder;
    inapi_ce_mapbuilder = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_mapbuilder(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_mapbuilder *intern;
    zval *tmp;

    intern = (php_mapbuilder*)emalloc(sizeof(php_mapbuilder));
    memset(intern, 0, sizeof(php_mapbuilder));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_mapbuilder,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_mapbuilder(void *object TSRMLS_DC) {
    php_mapbuilder *builder = (php_mapbuilder*)object;
    efree(builder);
}

/*
 * VecBuilder Class
 */

zend_class_entry *inapi_ce_vecbuilder;

static zend_function_entry vecbuilder_methods[] = {
    PHP_ME(VecBuilder, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(VecBuilder, push_str, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(VecBuilder, push_bool, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(VecBuilder, push_vec, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(VecBuilder, push_map, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_vecbuilder(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\VecBuilder", vecbuilder_methods);
    ce.create_object = create_php_vecbuilder;
    inapi_ce_vecbuilder = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_vecbuilder(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_vecbuilder *intern;
    zval *tmp;

    intern = (php_vecbuilder*)emalloc(sizeof(php_vecbuilder));
    memset(intern, 0, sizeof(php_vecbuilder));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_vecbuilder,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_vecbuilder(void *object TSRMLS_DC) {
    php_vecbuilder *builder = (php_vecbuilder*)object;
    efree(builder);
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
    zval *zbuilder;
    php_mapbuilder *mbuilder;
    php_vecbuilder *vbuilder;
    int fd;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &zbuilder) == FAILURE) {
        return;
    }

    intern = (php_template*)zend_object_store_get_object(getThis() TSRMLS_CC);

    switch (Z_TYPE_P(zbuilder)) {
        case IS_OBJECT:
            if (instanceof_function(Z_OBJCE_P(zbuilder), inapi_ce_mapbuilder TSRMLS_CC)) {
                mbuilder = (php_mapbuilder*)zend_object_store_get_object(zbuilder TSRMLS_CC);
                fd = template_render_map(intern->template, mbuilder->builder);
            }
            else if (instanceof_function(Z_OBJCE_P(zbuilder), inapi_ce_vecbuilder TSRMLS_CC)) {
                vbuilder = (php_vecbuilder*)zend_object_store_get_object(zbuilder TSRMLS_CC);
                fd = template_render_vec(intern->template, vbuilder->builder);
            } else {
                zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\MapBuilder", 1001 TSRMLS_CC);
                return;
            }
            break;

        default:
            zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\MapBuilder", 1001 TSRMLS_CC);
            return;
    }

    if (fd == 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    RETURN_LONG(fd);
}

/*
 * MapBuilder Methods
 */

PHP_METHOD(MapBuilder, __construct) {
    php_mapbuilder *intern;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    intern = (php_mapbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    MapBuilder *builder = map_new();

    if (!builder) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->builder = builder;
}

PHP_METHOD(MapBuilder, insert_str) {
    php_mapbuilder *intern;
    char *key, *value;
    int key_len, value_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "ss", &key, &key_len, &value, &value_len) == FAILURE) {
        return;
    }

    intern = (php_mapbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rc = map_insert_str(intern->builder, key, value);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(MapBuilder, insert_bool) {
    php_mapbuilder *intern;
    char *key;
    int key_len;
    bool *value;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sb", &key, &key_len, &value) == FAILURE) {
        return;
    }

    intern = (php_mapbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rc = map_insert_bool(intern->builder, key, value);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(MapBuilder, insert_vec) {
    php_mapbuilder *intern;
    char *key;
    int key_len;
    zval *value;
    php_vecbuilder *builder;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sz", &key, &key_len, &value) == FAILURE) {
        return;
    }

    intern = (php_mapbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    switch (Z_TYPE_P(value)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(value), inapi_ce_vecbuilder TSRMLS_CC)) {
                zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\VecBuilder", 1001 TSRMLS_CC);
                return;
            }

            builder = (php_vecbuilder*)zend_object_store_get_object(value TSRMLS_CC);
            break;

        default:
            zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\VecBuilder", 1001 TSRMLS_CC);
            return;
    }

    int rc = map_insert_vec(intern->builder, key, builder->builder);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(MapBuilder, insert_map) {
    php_mapbuilder *intern;
    char *key;
    int key_len;
    zval *value;
    php_mapbuilder *builder;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sz", &key, &key_len, &value) == FAILURE) {
        return;
    }

    intern = (php_mapbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    switch (Z_TYPE_P(value)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(value), inapi_ce_mapbuilder TSRMLS_CC)) {
                zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\MapBuilder", 1001 TSRMLS_CC);
                return;
            }

            builder = (php_mapbuilder*)zend_object_store_get_object(value TSRMLS_CC);
            break;

        default:
            zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\MapBuilder", 1001 TSRMLS_CC);
            return;
    }

    int rc = map_insert_map(intern->builder, key, builder->builder);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

/*
 * VecBuilder Methods
 */

PHP_METHOD(VecBuilder, __construct) {
    php_vecbuilder *intern;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    intern = (php_vecbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    VecBuilder *builder = vec_new();

    if (!builder) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->builder = builder;
}

PHP_METHOD(VecBuilder, push_str) {
    php_vecbuilder *intern;
    char *value;
    int value_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &value, &value_len) == FAILURE) {
        return;
    }

    intern = (php_vecbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rc = vec_push_str(intern->builder, value);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(VecBuilder, push_bool) {
    php_vecbuilder *intern;
    bool *value;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "b", &value) == FAILURE) {
        return;
    }

    intern = (php_vecbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rc = vec_push_bool(intern->builder, value);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(VecBuilder, push_vec) {
    php_vecbuilder *intern;
    zval *value;
    php_vecbuilder *builder;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &value) == FAILURE) {
        return;
    }

    intern = (php_vecbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    switch (Z_TYPE_P(value)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(value), inapi_ce_vecbuilder TSRMLS_CC)) {
                zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\VecBuilder", 1001 TSRMLS_CC);
                return;
            }

            builder = (php_vecbuilder*)zend_object_store_get_object(value TSRMLS_CC);
            break;

        default:
            zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\VecBuilder", 1001 TSRMLS_CC);
            return;
    }

    int rc = vec_push_vec(intern->builder, builder->builder);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(VecBuilder, push_map) {
    php_vecbuilder *intern;
    zval *value;
    php_mapbuilder *builder;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &value) == FAILURE) {
        return;
    }

    intern = (php_vecbuilder*)zend_object_store_get_object(getThis() TSRMLS_CC);

    switch (Z_TYPE_P(value)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(value), inapi_ce_mapbuilder TSRMLS_CC)) {
                zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\MapBuilder", 1001 TSRMLS_CC);
                return;
            }

            builder = (php_mapbuilder*)zend_object_store_get_object(value TSRMLS_CC);
            break;

        default:
            zend_throw_exception(inapi_ce_template_exception, "The first argument must be an instance of Intecture\\MapBuilder", 1001 TSRMLS_CC);
            return;
    }

    int rc = vec_push_map(intern->builder, builder->builder);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_template_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}
