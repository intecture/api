/*
 Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "command.h"
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
 * Command Class
 */

zend_class_entry *inapi_ce_command;

static zend_function_entry command_methods[] = {
  PHP_ME(Command, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
  PHP_ME(Command, exec, NULL, ZEND_ACC_PUBLIC)
  {NULL, NULL, NULL}
};

void inapi_init_command(TSRMLS_D) {
  zend_class_entry ce;

  INIT_CLASS_ENTRY(ce, "Intecture\\Command", command_methods);
  ce.create_object = create_php_command;
  inapi_ce_command = zend_register_internal_class(&ce TSRMLS_CC);
}

zend_object_value create_php_command(zend_class_entry *class_type TSRMLS_DC) {
  zend_object_value retval;
  php_command  *intern;
  zval *tmp;

  intern = (php_command*)emalloc(sizeof(php_command));
  memset(intern, 0, sizeof(php_command));

  zend_object_std_init(&intern->std, class_type TSRMLS_CC);
  object_properties_init(&intern->std, class_type);

  retval.handle = zend_objects_store_put(
      intern,
      (zend_objects_store_dtor_t) zend_objects_destroy_object,
      free_php_command,
      NULL TSRMLS_CC
  );
  retval.handlers = zend_get_std_object_handlers();

  return retval;
}

void free_php_command(void *object TSRMLS_DC) {
    php_command *command = (php_command*)object;
    if (command->command) {
        int rc = command_free(command->command);
        assert(rc == 0);
    }
    efree(command);
}

/*
 * Exception Class
 */

zend_class_entry *inapi_ce_command_exception;

void inapi_init_command_exception(TSRMLS_D) {
  zend_class_entry e;

  INIT_CLASS_ENTRY(e, "Intecture\\CommandException", NULL);
  inapi_ce_command_exception = zend_register_internal_class_ex(&e, (zend_class_entry*)zend_exception_get_default(TSRMLS_C), NULL TSRMLS_CC);
}

/*
 * Command Methods
 */

PHP_METHOD(Command, __construct) {
    php_command *intern;
    char *cmd;
    int cmd_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &cmd, &cmd_len) == FAILURE) {
        return;
    }

    intern = (php_command*)zend_object_store_get_object(getThis() TSRMLS_CC);

    Command *command = command_new(cmd);

    if (!command) {
        zend_throw_exception(inapi_ce_command_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    intern->command = command;
}

PHP_METHOD(Command, exec) {
    php_command *intern;
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    intern = (php_command*)zend_object_store_get_object(getThis() TSRMLS_CC);

    int rtn = get_check_host(phost, &host TSRMLS_CC);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_command_exception, "The first argument must be an instance of Intecture\\Host", 1000 TSRMLS_CC);
        return;
    }

    CommandResult *result = command_exec(intern->command, host->host);

    if (!result) {
        zend_throw_exception(inapi_ce_command_exception, geterr(), 1000 TSRMLS_CC);
        return;
    }

    array_init(return_value);
    add_assoc_long(return_value, "exit_code", result->exit_code);
    add_assoc_string(return_value, "stdout", result->stdout, 1);
    add_assoc_string(return_value, "stderr", result->stderr, 1);

    rtn = command_result_free(result);
    if (rtn != 0) {
        zend_throw_exception(inapi_ce_command_exception, "Could not free internal CommandResult struct", 1001 TSRMLS_CC);
        return;
    }
}
