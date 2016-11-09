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
 * Directory Class
 */

zend_class_entry *inapi_ce_directory;

static zend_function_entry directory_methods[] = {
    PHP_ME(Directory, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Directory, exists, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Directory, create, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Directory, delete, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Directory, mv, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Directory, get_owner, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Directory, set_owner, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Directory, get_mode, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Directory, set_mode, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

void inapi_init_directory(TSRMLS_D) {
    zend_class_entry ce;

    INIT_CLASS_ENTRY(ce, "Intecture\\Directory", directory_methods);
    ce.create_object = create_php_directory;
    inapi_ce_directory = zend_register_internal_class(&ce TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_directory, "OPT_DO_RECURSIVE", 16, OPT_DO_RECURSIVE TSRMLS_CC);
}

zend_object_value create_php_directory(zend_class_entry *class_type TSRMLS_DC) {
    zend_object_value retval;
    php_directory  *intern;
    zval *tmp;

    intern = (php_directory*)emalloc(sizeof(php_directory));
    memset(intern, 0, sizeof(php_directory));

    zend_object_std_init(&intern->std, class_type TSRMLS_CC);
    object_properties_init(&intern->std, class_type);

    retval.handle = zend_objects_store_put(
        intern,
        (zend_objects_store_dtor_t) zend_objects_destroy_object,
        free_php_directory,
        NULL TSRMLS_CC
    );
    retval.handlers = zend_get_std_object_handlers();

    return retval;
}

void free_php_directory(void *object TSRMLS_DC) {
    php_directory *directory = (php_directory*)object;
    if (directory->directory) {
        int rc = directory_free(directory->directory);
        assert(rc == 0);
    }
    efree(directory);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_directory_exception;

void inapi_init_directory_exception(TSRMLS_D) {
    zend_class_entry e;

    INIT_CLASS_ENTRY(e, "Intecture\\DirectoryException", NULL);
    inapi_ce_directory_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Directory Methods
 */

PHP_METHOD(Directory, __construct) {
    php_directory *intern;
    zval *phost;
    php_host *host;
    char *path;
    int path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &path, &path_len) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    Directory *directory = directory_new(host->host, path);

    if (!directory) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->directory = directory;
}

PHP_METHOD(Directory, exists) {
    php_directory *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int result = directory_exists(intern->directory, host->host);

    if (result < 0) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
    }
    else if (result == 1) {
        RETURN_TRUE;
    } else {
        RETURN_FALSE;
    }
}

PHP_METHOD(Directory, create) {
    php_directory *intern;
    zval *phost;
    php_host *host;
    zval *opts = NULL;
    zval **data;
    HashTable *arr_hash;
    HashPosition pointer;
    int array_count;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &phost, &opts) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    DirectoryOpts c_opts = { .do_recursive = false };

    if (opts != NULL) {
        arr_hash = Z_ARRVAL_P(opts);
        array_count = zend_hash_num_elements(arr_hash);

        for (zend_hash_internal_pointer_reset_ex(arr_hash, &pointer);
             zend_hash_get_current_data_ex(arr_hash, (void**) &data, &pointer) == SUCCESS;
             zend_hash_move_forward_ex(arr_hash, &pointer)) {
            switch (Z_LVAL_PP(data)) {
                case OPT_DO_RECURSIVE:
                    c_opts.do_recursive = true;
                    break;
                default:
                    zend_throw_exception(inapi_ce_directory_exception, "Invalid option key - must be Directory constant", 1001 TSRMLS_CC);
                    break;
            }
        }
    }

    int rc = directory_create(intern->directory, host->host, &c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, delete) {
    php_directory *intern;
    zval *phost;
    php_host *host;
    zval *opts = NULL;
    zval **data;
    HashTable *arr_hash;
    HashPosition pointer;
    int array_count;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z|a", &phost, &opts) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    DirectoryOpts c_opts = { .do_recursive = false };

    if (opts != NULL) {
        arr_hash = Z_ARRVAL_P(opts);
        array_count = zend_hash_num_elements(arr_hash);

        for (zend_hash_internal_pointer_reset_ex(arr_hash, &pointer);
             zend_hash_get_current_data_ex(arr_hash, (void**) &data, &pointer) == SUCCESS;
             zend_hash_move_forward_ex(arr_hash, &pointer)) {
            switch (Z_LVAL_PP(data)) {
                case OPT_DO_RECURSIVE:
                    c_opts.do_recursive = true;
                    break;
                default:
                    zend_throw_exception(inapi_ce_directory_exception, "Invalid option key - must be Directory constant", 1001 TSRMLS_CC);
                    break;
            }
        }
    }

    int rc = directory_delete(intern->directory, host->host, &c_opts);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, mv) {
    php_directory *intern;
    zval *phost;
    php_host *host;
    char *new_path;
    int new_path_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zs", &phost, &new_path, &new_path_len) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = directory_mv(intern->directory, host->host, new_path);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, get_owner) {
    php_directory *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    FileOwner *owner = directory_get_owner(intern->directory, host->host);

    if (!owner) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    array_init(return_value);
    add_assoc_string(return_value, "user_name", owner->user_name, 1);
    add_assoc_long(return_value, "user_uid", owner->user_uid);
    add_assoc_string(return_value, "group_name", owner->group_name, 1);
    add_assoc_long(return_value, "group_gid", owner->group_gid);
}

PHP_METHOD(Directory, set_owner) {
    php_directory *intern;
    zval *phost;
    php_host *host;
    char *user, *group;
    int user_len, group_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zss", &phost, &user, &user_len, &group, &group_len) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = directory_set_owner(intern->directory, host->host, user, group);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

PHP_METHOD(Directory, get_mode) {
    php_directory *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int16_t mode = directory_get_mode(intern->directory, host->host);

    if (mode < 0) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    RETURN_LONG(mode);
}

PHP_METHOD(Directory, set_mode) {
    php_directory *intern;
    zval *phost;
    php_host *host;
    long mode;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "zl", &phost, &mode) == FAILURE) {
        return;
    }

    intern = (php_directory*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_directory_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    int rc = directory_set_mode(intern->directory, host->host, mode);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_directory_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}
