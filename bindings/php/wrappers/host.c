/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

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
 * Host Class
 */

zend_class_entry *inapi_ce_host;

static zend_function_entry host_methods[] = {
  PHP_ME(Host, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
  PHP_ME(Host, connect, NULL, ZEND_ACC_PUBLIC)
  {NULL, NULL, NULL}
};

void inapi_init_host(TSRMLS_D) {
  zend_class_entry ce;

  INIT_CLASS_ENTRY(ce, "Intecture\\Host", host_methods);
  ce.create_object = create_php_host;
  inapi_ce_host = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_host(zend_class_entry *class_type TSRMLS_DC) {
  zend_object_value retval;
  php_host  *intern;
  zval *tmp;

  intern = (php_host*)emalloc(sizeof(php_host));
  memset(intern, 0, sizeof(php_host));

  zend_object_std_init(&intern->std, class_type TSRMLS_CC);
  object_properties_init(&intern->std, class_type);

  retval.handle = zend_objects_store_put(
      intern,
      (zend_objects_store_dtor_t) zend_objects_destroy_object,
      free_php_host,
      NULL TSRMLS_CC
  );
  retval.handlers = zend_get_std_object_handlers();

  return retval;
}

void free_php_host(void *object TSRMLS_DC) {
  php_host *host = (php_host*)object;
  host_close(host->host);
  efree(host);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_host_exception;

void inapi_init_host_exception(TSRMLS_D) {
  zend_class_entry e;

  INIT_CLASS_ENTRY(e, "Intecture\\HostException", NULL);
  inapi_ce_host_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Host Methods
 */

PHP_METHOD(Host, __construct) {
    php_host *intern;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    intern = (php_host*)zend_object_store_get_object(getThis() TSRMLS_CC);

    Host *host = host_new();

    if (!host) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->host = host;
}

PHP_METHOD(Host, connect) {
	php_host *intern;
	char *hostname;
	int hostname_len;
	long api_port, upload_port;

	if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "sll", &hostname, &hostname_len, &api_port, &upload_port) == FAILURE) {
		return;
	}

	intern = (php_host*)zend_object_store_get_object(getThis() TSRMLS_CC);

	int rc = host_connect(intern->host, hostname, api_port, upload_port);

    if (rc != 0) {
        zend_throw_exception(inapi_ce_host_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }
}

int get_check_host(zval *phost, php_host **host TSRMLS_DC) {
    switch (Z_TYPE_P(phost)) {
        case IS_OBJECT:
            if (!instanceof_function(Z_OBJCE_P(phost), inapi_ce_host TSRMLS_CC)) {
                return 1;
            }
            break;

        default:
            return 1;
            break;
    }

    *host = (php_host *)zend_object_store_get_object(phost TSRMLS_CC);

    return 0;
}
