/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "file.h"
#include "host.h"
#include <zend_exceptions.h>

extern zend_class_entry *inapi_ce_host, *inapi_ce_file_ex;

static inline php_file * php_file_fetch_object(zend_object *obj) {
      return (php_file *)((char *)obj - XtOffsetOf(php_file, std));
}

#define Z_FILE_OBJ_P(zv) php_file_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(File, __construct) {
    zval *phost;
    php_host *host;
    char *path;
    size_t path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zp", &phost, &path, &path_len) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    File *file = file_new(host->host, path);

    if (!file) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    intern->file = file;
}

PHP_METHOD(File, exists) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    bool *exists = file_exists(intern->file, host->host);

    if (!exists) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    if (*exists == true) {
        RETURN_TRUE;
    } else {
        RETURN_FALSE;
    }
}

PHP_METHOD(File, upload) {
    zval *phost;
    php_host *host;
    char *path;
    size_t path_len;
    zval *opts = NULL;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zp|a", &phost, &path, &path_len, &opts) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    FileOptions c_opts;
    int rc = parse_opts(opts, &c_opts TSRMLS_CC);
    if (rc != 0) {
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    rc = file_upload(intern->file, host->host, path, &c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, upload_file) {
    zval *phost;
    php_host *host;
    int fd;
    zval *opts = NULL;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zl|a", &phost, &fd, &opts) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    FileOptions c_opts;
    int rc = parse_opts(opts, &c_opts TSRMLS_CC);
    if (rc != 0) {
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    rc = file_upload_file(intern->file, host->host, fd, &c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, delete) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    int rc = file_delete(intern->file, host->host);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, mv) {
    zval *phost;
    php_host *host;
    char *new_path;
    size_t new_path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zp", &phost, &new_path, &new_path_len) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    int rc = file_mv(intern->file, host->host, new_path);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, copy) {
    zval *phost;
    php_host *host;
    char *new_path;
    size_t new_path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zp", &phost, &new_path, &new_path_len) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    int rc = file_copy(intern->file, host->host, new_path);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, get_owner) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    FileOwner *owner = file_get_owner(intern->file, host->host);

    if (!owner) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    array_init(return_value);
    add_assoc_string(return_value, "user_name", owner->user_name);
    add_assoc_long(return_value, "user_uid", owner->user_uid);
    add_assoc_string(return_value, "group_name", owner->group_name);
    add_assoc_long(return_value, "group_gid", owner->group_gid);
}

PHP_METHOD(File, set_owner) {
    zval *phost;
    php_host *host;
    char *user, *group;
    size_t user_len, group_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zss", &phost, &user, &user_len, &group, &group_len) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    int rc = file_set_owner(intern->file, host->host, user, group);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, get_mode) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    uint16_t *mode = file_get_mode(intern->file, host->host);

    if (!mode) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }

    RETURN_LONG(*mode);
}

PHP_METHOD(File, set_mode) {
    zval *phost;
    php_host *host;
    long mode;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zl", &phost, &mode) == FAILURE) {
        return;
    }

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_ex, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    php_file *intern = Z_FILE_OBJ_P(getThis());

    int rc = file_set_mode(intern->file, host->host, mode);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_ex, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

int parse_opts(zval *opts, FileOptions *fopts TSRMLS_DC) {
    zval *zv;
    zend_string *zk;
    ulong i;

    fopts->backup_existing = NULL;
    fopts->chunk_size = NULL;

    if (opts) {
        ZEND_HASH_FOREACH_KEY_VAL(Z_ARRVAL_P(opts), i, zk, zv) {
            if (zk) { // HASH_KEY_IS_STRING
                zend_throw_exception(inapi_ce_file_ex, "Invalid option key - must be File constant", 1001 TSRMLS_CC);
                return 1;
            } else {
                switch (i) {
                    case OPT_BACKUP_EXISTING:
                        fopts->backup_existing = Z_STRVAL_P(zv);
                        break;
                    case OPT_CHUNK_SIZE:
                        fopts->chunk_size = (unsigned long long *) Z_LVAL_P(zv);
                    default:
                        zend_throw_exception(inapi_ce_file_ex, "Invalid option key - must be File constant", 1001 TSRMLS_CC);
                        return 1;
                }
            }
        } ZEND_HASH_FOREACH_END();
    }

    return 0;
}
