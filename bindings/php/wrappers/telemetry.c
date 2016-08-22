/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "host.h"
#include "telemetry.h"
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
 * Telemetry Class
 */

zend_class_entry *inapi_ce_telemetry;

static zend_function_entry telemetry_methods[] = {
  PHP_ME(Telemetry, __construct, NULL, ZEND_ACC_PRIVATE|ZEND_ACC_CTOR)
  PHP_ME(Telemetry, load, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
  PHP_ME(Telemetry, get, NULL, ZEND_ACC_PUBLIC)
  {NULL, NULL, NULL}
};

void inapi_init_telemetry(TSRMLS_D) {
  zend_class_entry ce;

  INIT_CLASS_ENTRY(ce, "Intecture\\Telemetry", telemetry_methods);
  ce.create_object = create_php_telemetry;
  inapi_ce_telemetry = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_telemetry(zend_class_entry *class_type TSRMLS_DC) {
  zend_object_value retval;
  php_telemetry  *intern;
  zval *tmp;

  intern = (php_telemetry*)emalloc(sizeof(php_telemetry));
  memset(intern, 0, sizeof(php_telemetry));

  zend_object_std_init(&intern->std, class_type TSRMLS_CC);
  object_properties_init(&intern->std, class_type);

  retval.handle = zend_objects_store_put(
	  intern,
	  (zend_objects_store_dtor_t) zend_objects_destroy_object,
	  free_php_telemetry,
	  NULL TSRMLS_CC
  );
  retval.handlers = zend_get_std_object_handlers();

  return retval;
}

void free_php_telemetry(void *object TSRMLS_DC) {
  php_telemetry *telemetry = (php_telemetry*)object;
  efree(telemetry);
}

zval* telemetry_to_array(Telemetry *telemetry) {
    zval *cpu, *os, *fs, *net, *ztelemetry;
    ALLOC_INIT_ZVAL(cpu);
    ALLOC_INIT_ZVAL(os);
    ALLOC_INIT_ZVAL(fs);
    ALLOC_INIT_ZVAL(net);
    ALLOC_INIT_ZVAL(ztelemetry);

    // Telemetry
    array_init(ztelemetry);
    add_assoc_string(ztelemetry, "hostname", telemetry->hostname, 1);
    add_assoc_long(ztelemetry, "memory", telemetry->memory/1024/1024);

    // CPU
    array_init(cpu);
    add_assoc_string(cpu, "vendor", telemetry->cpu.vendor, 1);
    add_assoc_string(cpu, "brand_string", telemetry->cpu.brand_string, 1);
    add_assoc_long(cpu, "cores", telemetry->cpu.cores);
    add_assoc_zval(ztelemetry, "cpu", cpu);

    // FS
    array_init(fs);
    int f = 0;
    for (f = 0; f < telemetry->fs.length; f++) {
        add_next_index_zval(fs, fsmount_to_array(&telemetry->fs.ptr[f]));
    }
    add_assoc_zval(ztelemetry, "fs", fs);

    // Netif
    array_init(net);
    int n = 0;
    for (n = 0; n < telemetry->net.length; n++) {
        add_next_index_zval(net, netif_to_array(&telemetry->net.ptr[n]));
    }
    add_assoc_zval(ztelemetry, "net", net);

    // OS
    array_init(os);
    add_assoc_string(os, "arch", telemetry->os.arch, 1);
    add_assoc_string(os, "family", telemetry->os.family, 1);
    add_assoc_string(os, "platform", telemetry->os.platform, 1);
    add_assoc_string(os, "version", telemetry->os.version, 1);
    add_assoc_zval(ztelemetry, "os", os);

    return ztelemetry;
}

zval* fsmount_to_array(FsMount *fsmount) {
    zval *zfsmount;

    ALLOC_INIT_ZVAL(zfsmount);
    array_init(zfsmount);

    add_assoc_string(zfsmount, "filesystem", fsmount->filesystem, 1);
    add_assoc_string(zfsmount, "mount", fsmount->mountpoint, 1);
    add_assoc_long(zfsmount, "size", fsmount->size);
    add_assoc_long(zfsmount, "used", fsmount->used);
    add_assoc_long(zfsmount, "available", fsmount->available);
    add_assoc_long(zfsmount, "capacity", fsmount->capacity * 100);
//    add_assoc_long(zfsmount, "inodes_used", fsmount->inodes_used);
//    add_assoc_long(zfsmount, "inodes_available", fsmount->inodes_available);
//    add_assoc_long(zfsmount, "inodes_capacity", fsmount->inodes_capacity * 100);

    return zfsmount;
}

zval* netif_to_array(Netif *netif) {
    zval *znetif, *inet4, *inet6;

    ALLOC_INIT_ZVAL(znetif);
    array_init(znetif);

    add_assoc_string(znetif, "interface", netif->interface, 1);

    if (netif->mac && netif->mac[0] != '\0') {
        add_assoc_string(znetif, "mac", netif->mac, 1);
    }

    if (netif->status && netif->status[0] != '\0') {
        add_assoc_string(znetif, "status", netif->status, 1);
    }

    if (netif->inet && netif->inet->address && netif->inet->address[0] != '\0') {
        ALLOC_INIT_ZVAL(inet4);
        array_init(inet4);

        add_assoc_string(inet4, "address", netif->inet->address, 1);
        add_assoc_string(inet4, "netmask", netif->inet->netmask, 1);
        add_assoc_zval(znetif, "inet", inet4);
    }

    if (netif->inet6 && netif->inet6->address && netif->inet6->address[0] != '\0') {
        ALLOC_INIT_ZVAL(inet6);
        array_init(inet6);

        add_assoc_string(inet6, "address", netif->inet6->address, 1);
        add_assoc_long(inet6, "prefixlen", netif->inet6->prefixlen);
        if (netif->inet6->scopeid) {
            add_assoc_string(inet6, "scope_id", netif->inet6->scopeid, 1);
        }
        add_assoc_zval(znetif, "inet6", inet6);
    }

    return znetif;
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_telemetry_exception;

void inapi_init_telemetry_exception(TSRMLS_D) {
  zend_class_entry e;

  INIT_CLASS_ENTRY(e, "Intecture\\TelemetryException", NULL);
  inapi_ce_telemetry_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Telemetry Methods
 */

PHP_METHOD(Telemetry, load) {
	php_telemetry *intern;
	zval *phost;
	php_host *host;

	if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
		return;
	}

	object_init_ex(return_value, inapi_ce_telemetry);
	intern = (php_telemetry *)zend_object_store_get_object(return_value TSRMLS_CC);

	int rtn = get_check_host(phost, &host TSRMLS_CC);
	if (rtn != 0) {
		zend_throw_exception(inapi_ce_telemetry_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
		return;
	}

    Telemetry *telemetry = telemetry_init(host->host);

	if (!telemetry) {
		zend_throw_exception(inapi_ce_telemetry_exception, geterr(), 1000 TSRMLS_CC);
		return;
	}

    intern->telemetry = telemetry_to_array(telemetry);

    // Free the original struct as we have a native zval
    // representation instead.
    telemetry_free(telemetry);
}

PHP_METHOD(Telemetry, __construct) {

}

PHP_METHOD(Telemetry, get) {
    php_telemetry *intern;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "") == FAILURE) {
        return;
    }

    intern = (php_telemetry*)zend_object_store_get_object(getThis() TSRMLS_CC);

    RETURN_ZVAL(intern->telemetry, 1, 0);
}
