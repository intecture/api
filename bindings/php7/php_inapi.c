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
#include "wrappers/directory.h"
#include "wrappers/file.h"
#include "wrappers/host.h"
#include "wrappers/package.h"
#include "wrappers/payload.h"
#include "wrappers/service.h"
#include "wrappers/template.h"
#include <zend_exceptions.h>

zend_class_entry *inapi_ce_command, *inapi_ce_command_ex,
                 *inapi_ce_directory, *inapi_ce_directory_ex,
                 *inapi_ce_file, *inapi_ce_file_ex,
                 *inapi_ce_host, *inapi_ce_host_ex,
                 *inapi_ce_package, *inapi_ce_package_ex,
                 *inapi_ce_payload, *inapi_ce_payload_ex,
                 *inapi_ce_service, *inapi_ce_service_ex, *inapi_ce_service_runnable,
                 *inapi_ce_template, *inapi_ce_template_ex;

zend_object_handlers inapi_command_handlers,
                     inapi_directory_handlers,
                     inapi_file_handlers,
                     inapi_host_handlers,
                     inapi_package_handlers,
                     inapi_payload_handlers,
                     inapi_service_handlers, inapi_service_runnable_handlers,
                     inapi_template_handlers;

/*
 * Command
 */
static zend_function_entry command_methods[] = {
  PHP_ME(Command, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
  PHP_ME(Command, exec, NULL, ZEND_ACC_PUBLIC)
  {NULL, NULL, NULL}
};

zend_object *inapi_command_create(zend_class_entry *ce) {
    php_command *intern = ecalloc(1, sizeof(php_command) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_command_handlers;

    return &intern->std;
}

void inapi_command_free(zend_object *object TSRMLS_DC) {
    php_command *command = (php_command *)((char *)object - XtOffsetOf(php_command, std));
    zend_object_std_dtor(object);
}

/*
 * Directory
 */
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

zend_object *inapi_directory_create(zend_class_entry *ce) {
    php_directory *intern = ecalloc(1, sizeof(php_directory) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_directory_handlers;

    return &intern->std;
}

void inapi_directory_free(zend_object *object TSRMLS_DC) {
    php_directory *directory = (php_directory *)((char *)object - XtOffsetOf(php_directory, std));
    zend_object_std_dtor(object);
}

/*
 * File
 */
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

zend_object *inapi_file_create(zend_class_entry *ce) {
    php_file *intern = ecalloc(1, sizeof(php_file) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_file_handlers;

    return &intern->std;
}

void inapi_file_free(zend_object *object TSRMLS_DC) {
    php_file *file = (php_file *)((char *)object - XtOffsetOf(php_file, std));
    zend_object_std_dtor(object);
}

/*
 * Host
 */
static zend_function_entry host_methods[] = {
  PHP_ME(Host, __construct, NULL, ZEND_ACC_PRIVATE|ZEND_ACC_CTOR)
  PHP_ME(Host, connect, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
  PHP_ME(Host, connect_endpoint, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
  PHP_ME(Host, connect_payload, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
  PHP_ME(Host, data, NULL, ZEND_ACC_PUBLIC)
  {NULL, NULL, NULL}
};

zend_object *inapi_host_create(zend_class_entry *ce) {
    php_host *intern = ecalloc(1, sizeof(php_host) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_host_handlers;

    return &intern->std;
}

void inapi_host_free(zend_object *object TSRMLS_DC) {
    php_host *host = (php_host *)((char *)object - XtOffsetOf(php_host, std));
    host_close(host->host);
    zval_dtor(&host->data);
    zend_object_std_dtor(object);
}

/*
 * Package
 */
static zend_function_entry package_methods[] = {
    PHP_ME(Package, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Package, is_installed, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Package, install, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Package, uninstall, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

zend_object *inapi_package_create(zend_class_entry *ce) {
    php_package *intern = ecalloc(1, sizeof(php_package) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_package_handlers;

    return &intern->std;
}

void inapi_package_free(zend_object *object TSRMLS_DC) {
    php_package *package = (php_package *)((char *)object - XtOffsetOf(php_package, std));
    zend_object_std_dtor(object);
}

/*
 * Payload
 */
static zend_function_entry payload_methods[] = {
    PHP_ME(Payload, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Payload, build, NULL, ZEND_ACC_PUBLIC)
    PHP_ME(Payload, run, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

zend_object *inapi_payload_create(zend_class_entry *ce) {
    php_payload *intern = ecalloc(1, sizeof(php_payload) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_payload_handlers;

    return &intern->std;
}

void inapi_payload_free(zend_object *object TSRMLS_DC) {
    php_payload *payload = (php_payload *)((char *)object - XtOffsetOf(php_payload, std));
    zend_object_std_dtor(object);
}

/*
 * Service
 */
static zend_function_entry service_methods[] = {
    PHP_ME(Service, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Service, action, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

zend_object *inapi_service_create(zend_class_entry *ce) {
    php_service *intern = ecalloc(1, sizeof(php_service) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_service_handlers;

    return &intern->std;
}

void inapi_service_free(zend_object *object TSRMLS_DC) {
    php_service *service = (php_service *)((char *)object - XtOffsetOf(php_service, std));
    zend_object_std_dtor(object);
}

/*
 * ServiceRunnable
 */
static zend_function_entry service_runnable_methods[] = {
    PHP_ME(ServiceRunnable, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    {NULL, NULL, NULL}
};

zend_object *inapi_service_runnable_create(zend_class_entry *ce) {
    php_service_runnable *intern = ecalloc(1, sizeof(php_service_runnable) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_service_runnable_handlers;

    return &intern->std;
}

void inapi_service_runnable_free(zend_object *object TSRMLS_DC) {
    php_service_runnable *runnable = (php_service_runnable *)((char *)object - XtOffsetOf(php_service_runnable, std));

    if (runnable->service_runnable.command) {
        efree(runnable->service_runnable.command);
    }

    if (runnable->service_runnable.service) {
        efree(runnable->service_runnable.service);
    }

    zend_object_std_dtor(object);
}

/*
 * Template
 */
static zend_function_entry template_methods[] = {
    PHP_ME(Template, __construct, NULL, ZEND_ACC_PUBLIC|ZEND_ACC_CTOR)
    PHP_ME(Template, render, NULL, ZEND_ACC_PUBLIC)
    {NULL, NULL, NULL}
};

zend_object *inapi_template_create(zend_class_entry *ce) {
    php_template *intern = ecalloc(1, sizeof(php_template) + zend_object_properties_size(ce));

    zend_object_std_init(&intern->std, ce);
    object_properties_init(&intern->std, ce);

    intern->std.handlers = &inapi_template_handlers;

    return &intern->std;
}

void inapi_template_free(zend_object *object TSRMLS_DC) {
    php_template *template = (php_template *)((char *)object - XtOffsetOf(php_template, std));
    zend_object_std_dtor(object);
}

PHP_MINIT_FUNCTION(inapi)
{
    zend_class_entry ce_command, ce_command_ex,
                     ce_directory, ce_directory_ex,
                     ce_file, ce_file_ex,
                     ce_host, ce_host_ex,
                     ce_package, ce_package_ex,
                     ce_payload, ce_payload_ex,
                     ce_service, ce_service_ex, ce_service_runnable,
                     ce_template, ce_template_ex;

    memcpy(&inapi_command_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_directory_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_file_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_host_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_package_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_payload_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_service_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_service_runnable_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));
    memcpy(&inapi_template_handlers, zend_get_std_object_handlers(), sizeof(zend_object_handlers));

    /*
     * Command
     */
    INIT_CLASS_ENTRY(ce_command, "Intecture\\Command", command_methods);
    ce_command.create_object = inapi_command_create;

    inapi_command_handlers.offset = XtOffsetOf(php_command, std);
    inapi_command_handlers.free_obj = inapi_command_free;

    inapi_ce_command = zend_register_internal_class(&ce_command);

    INIT_CLASS_ENTRY(ce_command_ex, "Intecture\\CommandException", NULL);
    inapi_ce_command_ex = zend_register_internal_class_ex(&ce_command_ex, zend_exception_get_default());

    /*
     * Directory
     */
    INIT_CLASS_ENTRY(ce_directory, "Intecture\\Directory", directory_methods);
    ce_directory.create_object = inapi_directory_create;

    inapi_directory_handlers.offset = XtOffsetOf(php_directory, std);
    inapi_directory_handlers.free_obj = inapi_directory_free;

    inapi_ce_directory = zend_register_internal_class(&ce_directory);
    zend_declare_class_constant_long(inapi_ce_directory, "OPT_DO_RECURSIVE", 16, OPT_DO_RECURSIVE TSRMLS_CC);

    INIT_CLASS_ENTRY(ce_directory_ex, "Intecture\\DirectoryException", NULL);
    inapi_ce_directory_ex = zend_register_internal_class_ex(&ce_directory_ex, zend_exception_get_default());

    /*
     * File
     */
    INIT_CLASS_ENTRY(ce_file, "Intecture\\File", file_methods);
    ce_file.create_object = inapi_file_create;

    inapi_file_handlers.offset = XtOffsetOf(php_file, std);
    inapi_file_handlers.free_obj = inapi_file_free;

    inapi_ce_file = zend_register_internal_class(&ce_file);
    zend_declare_class_constant_long(inapi_ce_file, "OPT_BACKUP_EXISTING", 19, OPT_BACKUP_EXISTING TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_file, "OPT_CHUNK_SIZE", 14, OPT_CHUNK_SIZE TSRMLS_CC);

    INIT_CLASS_ENTRY(ce_file_ex, "Intecture\\FileException", NULL);
    inapi_ce_file_ex = zend_register_internal_class_ex(&ce_file_ex, zend_exception_get_default());

    /*
     * Host
     */
    INIT_CLASS_ENTRY(ce_host, "Intecture\\Host", host_methods);
    ce_host.create_object = inapi_host_create;

    inapi_host_handlers.offset = XtOffsetOf(php_host, std);
    inapi_host_handlers.free_obj = inapi_host_free;

    inapi_ce_host = zend_register_internal_class(&ce_host);

    INIT_CLASS_ENTRY(ce_host_ex, "Intecture\\HostException", NULL);
    inapi_ce_host_ex = zend_register_internal_class_ex(&ce_host_ex, zend_exception_get_default());

    /*
     * Package
     */
    INIT_CLASS_ENTRY(ce_package, "Intecture\\Package", package_methods);
    ce_package.create_object = inapi_package_create;

    inapi_package_handlers.offset = XtOffsetOf(php_package, std);
    inapi_package_handlers.free_obj = inapi_package_free;

    inapi_ce_package = zend_register_internal_class(&ce_package);

    INIT_CLASS_ENTRY(ce_package_ex, "Intecture\\PackageException", NULL);
    inapi_ce_package_ex = zend_register_internal_class_ex(&ce_package_ex, zend_exception_get_default());

    /*
     * Payload
     */
    INIT_CLASS_ENTRY(ce_payload, "Intecture\\Payload", payload_methods);
    ce_payload.create_object = inapi_payload_create;

    inapi_payload_handlers.offset = XtOffsetOf(php_payload, std);
    inapi_payload_handlers.free_obj = inapi_payload_free;

    inapi_ce_payload = zend_register_internal_class(&ce_payload);

    INIT_CLASS_ENTRY(ce_payload_ex, "Intecture\\PayloadException", NULL);
    inapi_ce_payload_ex = zend_register_internal_class_ex(&ce_payload_ex, zend_exception_get_default());

    /*
     * Service
     */
    INIT_CLASS_ENTRY(ce_service, "Intecture\\Service", service_methods);
    ce_service.create_object = inapi_service_create;

    inapi_service_handlers.offset = XtOffsetOf(php_service, std);
    inapi_service_handlers.free_obj = inapi_service_free;

    inapi_ce_service = zend_register_internal_class(&ce_service);

    INIT_CLASS_ENTRY(ce_service_ex, "Intecture\\ServiceException", NULL);
    inapi_ce_service_ex = zend_register_internal_class_ex(&ce_service_ex, zend_exception_get_default());

    /*
     * ServiceRunnable
     */
    INIT_CLASS_ENTRY(ce_service_runnable, "Intecture\\ServiceRunnable", service_runnable_methods);
    ce_service_runnable.create_object = inapi_service_runnable_create;

    inapi_service_runnable_handlers.offset = XtOffsetOf(php_service_runnable, std);
    inapi_service_runnable_handlers.free_obj = inapi_service_runnable_free;

    inapi_ce_service_runnable = zend_register_internal_class(&ce_service_runnable);
    zend_declare_class_constant_long(inapi_ce_service_runnable, "COMMAND", 7, RUNNABLE_COMMAND TSRMLS_CC);
    zend_declare_class_constant_long(inapi_ce_service_runnable, "SERVICE", 7, RUNNABLE_SERVICE TSRMLS_CC);

    /*
     * Template
     */
    INIT_CLASS_ENTRY(ce_template, "Intecture\\Template", template_methods);
    ce_template.create_object = inapi_template_create;

    inapi_template_handlers.offset = XtOffsetOf(php_template, std);
    inapi_template_handlers.free_obj = inapi_template_free;

    inapi_ce_template = zend_register_internal_class(&ce_template);

    INIT_CLASS_ENTRY(ce_template_ex, "Intecture\\TemplateException", NULL);
    inapi_ce_template_ex = zend_register_internal_class_ex(&ce_template_ex, zend_exception_get_default());

    return SUCCESS;
}

zend_module_entry inapi_module_entry = {
    STANDARD_MODULE_HEADER,
    PHP_INAPI_EXTNAME,     /* Extension name */
    NULL,                  /* Functions */
    PHP_MINIT(inapi),      /* Methods */
    NULL,                  /* MSHUTDOWN */
    NULL,                  /* RINIT */
    NULL,                  /* RSHUTDOWN */
    NULL,                  /* MINFO */
    PHP_INAPI_EXTVER,      /* Extension version */
    STANDARD_MODULE_PROPERTIES
};

#ifdef COMPILE_DL_INAPI
ZEND_GET_MODULE(inapi)
#endif
