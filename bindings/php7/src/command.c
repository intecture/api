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

extern zend_class_entry *inapi_ce_host, *inapi_ce_command_ex;

static inline php_command * php_command_fetch_object(zend_object *obj) {
      return (php_command *)((char *)obj - XtOffsetOf(php_command, std));
}

#define Z_CMD_OBJ_P(zv) php_command_fetch_object(Z_OBJ_P(zv));

PHP_METHOD(Command, __construct) {
    char *cmd;
    size_t cmd_len;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "s", &cmd, &cmd_len) == FAILURE) {
        return;
    }

    Command *command = command_new(cmd);

    if (!command) {
        zend_throw_exception(inapi_ce_command_ex, geterr(), 1000);
        return;
    }

    php_command *intern = Z_CMD_OBJ_P(getThis());

    intern->command = command;
}

PHP_METHOD(Command, exec) {
    zval *phost;
    php_host *host;

    if (zend_parse_parameters(ZEND_NUM_ARGS() TSRMLS_CC, "z", &phost) == FAILURE) {
        return;
    }

    host = check_host(phost TSRMLS_CC);
    if (!host) {
        zend_throw_exception(inapi_ce_command_ex, "The first argument must be an instance of Intecture\\Host", 1000);
        return;
    }

    php_command *intern = Z_CMD_OBJ_P(getThis());

    CommandResult *result = command_exec(intern->command, host->host);

    if (!result) {
        zend_throw_exception(inapi_ce_command_ex, geterr(), 1000);
        return;
    }

    array_init(return_value);
    add_assoc_long(return_value, "exit_code", result->exit_code);
    add_assoc_string(return_value, "stdout", result->stdout);
    add_assoc_string(return_value, "stderr", result->stderr);

    int rc = command_result_free(result);
    if (rc != 0) {
        zend_throw_exception(inapi_ce_command_ex, "Could not free internal CommandResult struct", 1001);
        return;
    }
}
