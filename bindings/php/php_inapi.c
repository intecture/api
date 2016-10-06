/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

#include "php_inapi.h"
#include "wrappers/command.h"
#include "wrappers/data.h"
#include "wrappers/directory.h"
#include "wrappers/file.h"
#include "wrappers/host.h"
#include "wrappers/package.h"
#include "wrappers/service.h"
#include "wrappers/telemetry.h"
#include "wrappers/template.h"

static zend_function_entry global_functions[] = {
    PHP_FE(data_open, NULL)
    {NULL, NULL, NULL}
};

PHP_MINIT_FUNCTION(inapi)
{
    inapi_init_host(TSRMLS_C);
    inapi_init_host_exception(TSRMLS_C);
    inapi_init_command(TSRMLS_C);
    inapi_init_command_exception(TSRMLS_C);
    inapi_init_data_exception(TSRMLS_C);
    inapi_init_directory(TSRMLS_C);
    inapi_init_directory_exception(TSRMLS_C);
    inapi_init_file(TSRMLS_C);
    inapi_init_file_exception(TSRMLS_C);
    inapi_init_package(TSRMLS_C);
    inapi_init_package_exception(TSRMLS_C);
    inapi_init_service(TSRMLS_C);
    inapi_init_service_exception(TSRMLS_C);
    inapi_init_service_runnable(TSRMLS_C);
    inapi_init_telemetry(TSRMLS_C);
    inapi_init_telemetry_exception(TSRMLS_C);
    inapi_init_template(TSRMLS_C);
    inapi_init_template_exception(TSRMLS_C);
    inapi_init_mapbuilder(TSRMLS_C);
    inapi_init_vecbuilder(TSRMLS_C);
    return SUCCESS;
}

zend_module_entry inapi_module_entry = {
#if ZEND_MODULE_API_NO >= 20010901
    STANDARD_MODULE_HEADER,
#endif
    PHP_INAPI_EXTNAME,     /* Extension name */
    global_functions,      /* Functions */
    PHP_MINIT(inapi),      /* Methods */
    NULL,                  /* MSHUTDOWN */
    NULL,                  /* RINIT */
    NULL,                  /* RSHUTDOWN */
    NULL,                  /* MINFO */
#if ZEND_MODULE_API_NO >= 20010901
    PHP_INAPI_EXTVER,      /* Extension version */
#endif
    STANDARD_MODULE_PROPERTIES
};

#ifdef COMPILE_DL_INAPI
ZEND_GET_MODULE(inapi)
#endif
