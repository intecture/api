/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT directory at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This directory may not be copied,
 modified, or distributed except according to those terms.
*/

#include "directory.h"
#include "host.h"
#include <zend_exceptions.h>

extern zend_class_entry *inapi_ce_host, *inapi_ce_directory_ex;

static inline php_directory * php_directory_fetch_object(zend_object *obj) {
      return (php_directory *)((char *)obj - XtOffsetOf(php_directory, std));
}

#define Z_DIR_OBJ_P(zv) php_directory_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(Directory, __construct) {
    zval *phost;
    php_host *host;
    char *path;
    size_t path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &path, &path_len) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    Directory *directory = directory_new(host->host, path);

    if (!directory) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    intern->directory = directory;
}

PHP_METHOD(Directory, exists) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    bool *result = directory_exists(intern->directory, host->host);

    if (!result) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    if (*result == true) {
        RETURN_TRUE;
    } else {
        RETURN_FALSE;
    }
}

PHP_METHOD(Directory, create) {
    zval *phost, *opts, *zv;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &phost, &opts) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    DirectoryOpts c_opts = { .do_recursive = false };

    if (opts) {
        HashTable *ht = Z_ARRVAL_P(opts);

        ZEND_HASH_FOREACH_VAL(ht, zv) {
            switch (Z_LVAL_P(zv)) {
                case OPT_DO_RECURSIVE:
                    c_opts.do_recursive = true;
                    break;
                default:
                    zend_throw_exception(inapi_ce_directory_ex, "Invalid option key - must be Directory constant", 1001 TSRMLS_CC);
                    break;
            }
        } ZEND_HASH_FOREACH_END();
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    int rc = directory_create(intern->directory, host->host, &c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, delete) {
    zval *phost, *opts, *zv;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &phost, &opts) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    DirectoryOpts c_opts = { .do_recursive = false };

    if (opts) {
        HashTable *ht = Z_ARRVAL_P(opts);

        ZEND_HASH_FOREACH_VAL(ht, zv) {
            switch (Z_LVAL_P(zv)) {
                case OPT_DO_RECURSIVE:
                    c_opts.do_recursive = true;
                    break;
                default:
                    zend_throw_exception(inapi_ce_directory_ex, "Invalid option key - must be Directory constant", 1001 TSRMLS_CC);
                    break;
            }
        } ZEND_HASH_FOREACH_END();
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    int rc = directory_delete(intern->directory, host->host, &c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, mv) {
    zval *phost;
    php_host *host;
    char *new_path;
    size_t new_path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &new_path, &new_path_len) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    int rc = directory_mv(intern->directory, host->host, new_path);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, get_owner) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    FileOwner *owner = directory_get_owner(intern->directory, host->host);

    if (!owner) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    array_init(return_value);
    add_assoc_string(return_value, "user_name", owner->user_name);
    add_assoc_long(return_value, "user_uid", owner->user_uid);
    add_assoc_string(return_value, "group_name", owner->group_name);
    add_assoc_long(return_value, "group_gid", owner->group_gid);
}

PHP_METHOD(Directory, set_owner) {
    zval *phost;
    php_host *host;
    char *user, *group;
    size_t user_len, group_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zss", &phost, &user, &user_len, &group, &group_len) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    int rc = directory_set_owner(intern->directory, host->host, user, group);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, get_mode) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    uint16_t *mode = directory_get_mode(intern->directory, host->host);

    if (!mode) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    RETURN_LONG(*mode);
}

PHP_METHOD(Directory, set_mode) {
    zval *phost;
    php_host *host;
    long mode;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zl", &phost, &mode) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_directory *intern = Z_DIR_OBJ_P(getThis());
    int rc = directory_set_mode(intern->directory, host->host, mode);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}
