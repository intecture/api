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
 * File Class
 */

zend_class_entry *inapi_ce_file;

static zend_function_entry file_methods[] = {
    PHP_ME(File, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(File, exists, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, upload, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, upload_file, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, delete, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, mv, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, copy, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, get_owner, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, set_owner, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, get_mode, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(File, set_mode, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_file(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\File", file_methods);
    ce.create_object = create_php_file;
    inapi_ce_file = zend_register_internal_class(&ce TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_file, "OPT_BACKUP_EXISTING", 19, OPT_BACKUP_EXISTING TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_file, "OPT_CHUNK_SIZE", 14, OPT_CHUNK_SIZE TSRMLS_CC);
}

zend_object_value create_php_file(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_file  *intern;
    zval *tmp;

    intern = (php_file*)emalloc(sizeof(php_file));
    memset(intern, 0, sizeof(php_file));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_file,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_file(void *object TSRMLS_DC) {
    php_file *file = (php_file*)object;
    if (file->file) {
        int rc = file_free(file->file);
        assert(rc == 0);
    }
    efree(file);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_file_exception;

void inapi_init_file_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\FileException", NULL);
    inapi_ce_file_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * File Methods
 */

PHP_METHOD(File, __construct) {
    php_file *intern;
    zval *phost;
    php_host *host;
    char *path;
    int path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &path, &path_len) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    File *file = file_new(host->host, path);

    if (!file) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->file = file;
}

PHP_METHOD(File, exists) {
    php_file *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int exists = file_exists(intern->file, host->host);

    if (exists < 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
    }
    else if (exists == 1) {
        RETURN_TRUE;
    } else {
        RETURN_FALSE;
    }
}

PHP_METHOD(File, upload) {
    php_file *intern;
    zval *phost;
    php_host *host;
    char *path;
    int path_len;
    zval *opts = NULL;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs|a", &phost, &path, &path_len, &opts) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rc = get_check_host(phost, &host TSRMLS_CC);
    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    FileOptions *c_opts = parse_opts(opts TSRMLS_CC);
    if (!c_opts) {
        return;
    }

    rc = file_upload(intern->file, host->host, path, c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, upload_file) {
    php_file *intern;
    zval *phost;
    php_host *host;
    long fd;
    zval *opts = NULL;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zl|a", &phost, &fd, &opts) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rc = get_check_host(phost, &host TSRMLS_CC);
    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    FileOptions *c_opts = parse_opts(opts TSRMLS_CC);
    if (!c_opts) {
        return;
    }

    rc = file_upload_file(intern->file, host->host, fd, c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, delete) {
    php_file *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = file_delete(intern->file, host->host);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, mv) {
    php_file *intern;
    zval *phost;
    php_host *host;
    char *new_path;
    int new_path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &new_path, &new_path_len) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = file_mv(intern->file, host->host, new_path);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, copy) {
    php_file *intern;
    zval *phost;
    php_host *host;
    char *new_path;
    int new_path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &new_path, &new_path_len) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = file_copy(intern->file, host->host, new_path);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, get_owner) {
    php_file *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    FileOwner *owner = file_get_owner(intern->file, host->host);

    if (!owner) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    array_init(return_value);
    add_assoc_string(return_value, "user_name", owner->user_name, 1);
    add_assoc_long(return_value, "user_uid", owner->user_uid);
    add_assoc_string(return_value, "group_name", owner->group_name, 1);
    add_assoc_long(return_value, "group_gid", owner->group_gid);
}

PHP_METHOD(File, set_owner) {
    php_file *intern;
    zval *phost;
    php_host *host;
    char *user, *group;
    int user_len, group_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zss", &phost, &user, &user_len, &group, &group_len) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = file_set_owner(intern->file, host->host, user, group);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(File, get_mode) {
    php_file *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int16_t mode = file_get_mode(intern->file, host->host);

    if (mode < 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    RETURN_LONG(mode);
}

PHP_METHOD(File, set_mode) {
    php_file *intern;
    zval *phost;
    php_host *host;
    long mode;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zl", &phost, &mode) == FAILURE) {
        return;
    }

    intern = (php_file*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_file_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = file_set_mode(intern->file, host->host, mode);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_file_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

FileOptions *parse_opts(zval *opts TSRMLS_DC) {
    zval **data;
    HashTable *arr_hash;
    HashPosition pointer;
    int array_count;

    FileOptions *c_opts = emalloc(sizeof(FileOptions));
    c_opts->backup_existing = NULL;
    c_opts->chunk_size = 0;

    if (opts != NULL) {
        arr_hash = Z_ARRVAL_P(opts);
        array_count = zend_hash_num_elements(arr_hash);

        for (zend_hash_internal_pointer_reset_ex(arr_hash, &pointer); zend_hash_get_current_data_ex(arr_hash, (void**) &data, &pointer) == SUCCESS; zend_hash_move_forward_ex(arr_hash, &pointer)) {
            char *key;
            unsigned int key_len;
            unsigned long index;

            if (zend_hash_get_current_key_ex(arr_hash, &key, &key_len, &index, 0, &pointer) == HASH_KEY_IS_LONG) {
                switch (index) {
                    case OPT_BACKUP_EXISTING:
                        c_opts->backup_existing = strdup(Z_STRVAL_PP(data));
                        break;
                    case OPT_CHUNK_SIZE:
                        c_opts->chunk_size = (unsigned long long)Z_LVAL_PP(data);
                    default:
                        zend_throw_exception(inapi_ce_file_exception, "Invalid option key - must be File constant", 1001 TSRMLS_CC);
                        return NULL;
                }
            } else {
                zend_throw_exception(inapi_ce_file_exception, "Invalid option key - must be File constant", 1001 TSRMLS_CC);
                return NULL;
            }
        }
    }

    return c_opts;
}
