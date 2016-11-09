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

extern zend_class_entry *inapi_ce_host, *inapi_ce_template_ex;

static inline php_template * php_template_fetch_object(zend_object *obj) {
      return (php_template *)((char *)obj - XtOffsetOf(php_template, std));
}

#define Z_TPL_OBJ_P(zv) php_template_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(Template, __construct) {
    char *path;
    size_t path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &path, &path_len) == FAILURE) {
        return;
    }

    Template *template = template_new(path);

    if (!template) {
        zend_throw_exception(inapi_ce_template_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    php_template *intern = Z_TPL_OBJ_P(getThis());

    intern->template = template;
}

PHP_METHOD(Template, render) {
    zval *data;
    int fd, rc;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "a", &data) == FAILURE) {
        return;
    }

    php_template *intern = Z_TPL_OBJ_P(getThis());

    HashTable *ht = Z_ARRVAL_P(data);

    if (array_is_hash(ht TSRMLS_CC)) {
        MapBuilder *hbuilder = map_new();
        rc = build_map(ht, hbuilder TSRMLS_CC);

        if (rc != 0) {
            zend_throw_exception(inapi_ce_template_ex, geterr(), 1000 TSRMLS_CC);
            return;
        }

        fd = template_render_map(intern->template, hbuilder);
    } else {
        VecBuilder *vbuilder = vec_new();
        rc = build_vec(ht, vbuilder TSRMLS_CC);

        if (rc != 0) {
            zend_throw_exception(inapi_ce_template_ex, geterr(), 1000 TSRMLS_CC);
            return;
        }

        fd = template_render_vec(intern->template, vbuilder);
    }

    if (fd == 0) {
        zend_throw_exception(inapi_ce_template_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    RETURN_LONG(fd);
}

bool array_is_hash(HashTable *ht TSRMLS_DC) {
    zend_string *zk;

    ZEND_HASH_FOREACH_STR_KEY(ht, zk) {
        if (zk) { // HASH_KEY_IS_STRING
            return true;
        }
    } ZEND_HASH_FOREACH_END();

    return false;
}

int build_map(HashTable *ht, MapBuilder *builder TSRMLS_DC) {
    zval *zv;
    zend_string *zk;
    ulong i;
    int rc;

    ZEND_HASH_FOREACH_KEY_VAL(ht, i, zk, zv) {
        char *key, intbuf[256], indexkey[256];

        if (zk) { // HASH_KEY_IS_STRING
            key = ZSTR_VAL(zk);
        }
        // Convert integer indexes to char
        else {
            sprintf(indexkey, "%lu", i);
            key = &indexkey[0];
        }

        if (Z_TYPE_P(zv) == IS_TRUE) {
            rc = map_insert_bool(builder, key, true);
        }
        else if (Z_TYPE_P(zv) == IS_FALSE) {
            rc = map_insert_bool(builder, key, false);
        }
        else if (Z_TYPE_P(zv) == IS_DOUBLE) {
            sprintf(intbuf, "%g", Z_DVAL_P(zv));
            rc = map_insert_str(builder, key, &intbuf[0]);
        }
        else if (Z_TYPE_P(zv) == IS_LONG) {
            sprintf(intbuf, "%li", Z_LVAL_P(zv));
            rc = map_insert_str(builder, key, &intbuf[0]);
        }
        else if (Z_TYPE_P(zv) == IS_STRING) {
            rc = map_insert_str(builder, key, Z_STRVAL_P(zv));
        }
        else if (Z_TYPE_P(zv) == IS_ARRAY) {
            HashTable *ht1 = Z_ARRVAL_P(zv);

            if (array_is_hash(ht1 TSRMLS_CC)) {
                MapBuilder *hbuilder = map_new();
                rc = build_map(ht1, hbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = map_insert_map(builder, key, hbuilder);
                }
            } else {
                VecBuilder *vbuilder = vec_new();
                rc = build_vec(ht1, vbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = map_insert_vec(builder, key, vbuilder);
                }
            }
        }
        else if (Z_TYPE_P(zv) == IS_NULL) {
            rc = map_insert_bool(builder, key, false);
        } else {
            zend_throw_exception(inapi_ce_template_ex, "Array value cannot be a resource or object", 1001 TSRMLS_CC);
            rc = -1;
        }

        if (rc != 0) {
            return rc;
        }
    } ZEND_HASH_FOREACH_END();

    return rc;
}

int build_vec(HashTable *ht, VecBuilder *builder TSRMLS_DC) {
    zval *zv;
    zend_string *zk;
    ulong i;
    int rc;

    ZEND_HASH_FOREACH_KEY_VAL(ht, i, zk, zv) {
        char intbuf[256];

        if (Z_TYPE_P(zv) == IS_TRUE) {
            rc = vec_push_bool(builder, true);
        }
        else if (Z_TYPE_P(zv) == IS_FALSE) {
            rc = vec_push_bool(builder, false);
        }
        else if (Z_TYPE_P(zv) == IS_DOUBLE) {
            sprintf(intbuf, "%g", Z_DVAL_P(zv));
            rc = vec_push_str(builder, &intbuf[0]);
        }
        else if (Z_TYPE_P(zv) == IS_LONG) {
            sprintf(intbuf, "%li", Z_LVAL_P(zv));
            rc = vec_push_str(builder, &intbuf[0]);
        }
        else if (Z_TYPE_P(zv) == IS_STRING) {
            rc = vec_push_str(builder, Z_STRVAL_P(zv));
        }
        else if (Z_TYPE_P(zv) == IS_ARRAY) {
            HashTable *ht1 = Z_ARRVAL_P(zv);

            if (array_is_hash(ht1 TSRMLS_CC)) {
                MapBuilder *hbuilder = map_new();
                rc = build_map(ht1, hbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = vec_push_map(builder, hbuilder);
                }
            } else {
                VecBuilder *vbuilder = vec_new();
                rc = build_vec(ht1, vbuilder TSRMLS_CC);
                if (rc == 0) {
                    rc = vec_push_vec(builder, vbuilder);
                }
            }
        }
        else if (Z_TYPE_P(zv) == IS_NULL) {
            rc = vec_push_bool(builder, false);
        } else {
            zend_throw_exception(inapi_ce_template_ex, "Array value cannot be a resource or object", 1001 TSRMLS_CC);
            rc = -1;
        }

        if (rc != 0) {
            return rc;
        }
    } ZEND_HASH_FOREACH_END();

    return rc;
}
