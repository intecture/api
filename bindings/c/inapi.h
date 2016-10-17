/*
 Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
 top-level directory of this distribution and at
 https://intecture.io/COPYRIGHT.

 Licensed under the Mozilla Public License 2.0 <LICENSE or
 https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
 modified, or distributed except according to those terms.
*/

/**
 * @file inapi.h
 * @author see AUTHORS
 * @version 0.2.1
 * @brief C headers for Intecture API
 *
 * Intecture API allows C programs to consume its library by exposing
 * functions via its FFI.
 */
#ifndef INAPI_H
#define INAPI_H 1

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

/**
 * @brief Retrieve the last error message generated and reset the
 * error global to null. It will return null if no error message was
 * recorded.
 */
extern char *geterr();

/**
 * @brief The host primitive for connecting to a managed host.
 */
typedef struct _Host {
    const char *hostname; /**< Hostname or IP of managed host */
    void *api_sock; /**< API socket */
    void *file_sock; /**< File upload socket */
    void *data; /**< Data for host, comprising data files and telemetry */
} Host;

/**
 * @brief Create a new Host connected to the endpoint specified in the
 *        data file. This function expects to find the following keys
 *        in the root namespace: "hostname", "api_port", "file_port".
 * @param path Path to the data file for this host.
 * @return A new Host struct.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 * @endcode
 */
extern Host *host_connect(const char *path);

/**
 * @brief Create a new Host connected to the specified endpoint. Note
 *        that this function does not load any user data.
 * @param hostname The IP address or hostname of your managed host.
 * @param api_port The port number that the Agent API service is listening on.
 * @param upload_port The port number that the Agent File Upload service is listening on.
 * @return Return code - zero on success, non-zero on error.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect_endpoint("example.com", 7101, 7102);
 * assert(host);
 * @endcode
 */
extern Host *host_connect_endpoint(const char *hostname, uint32_t api_port, uint32_t upload_port);

/**
 * @brief Close the connection to your managed host.
 * @param host The host connection you wish to close.
 * @return Return code - zero on success, non-zero on error.
 */
extern int host_close(Host *host);

/**
 * @brief Potential data types for a given `Value`.
 */
enum DataType {
    Null, /**< Null */
    Bool, /**< Boolean */
    Int64, /**< 64b signed integer */
    Uint64, /**< 64b unsigned integer */
    Float, /**< 64b floating point integer */
    String, /**< Char array */
    Array, /**< Array of `Value`s */
    Object, /**< A `Value` pointer to the object  */
};

/**
 * @brief Array of `Value` pointers
 */
typedef struct _ValueArray {
    void **ptr; /**< `Value`s */
    size_t length; /**< Size of array */
    size_t capacity; /**< Capacity of array */
} ValueArray;

/**
 * @brief Array of `Value` pointers
 */
typedef struct _ValueKeysArray {
    char **ptr; /**< Keys */
    size_t length; /**< Size of array */
    size_t capacity; /**< Capacity of array */
} ValueKeysArray;

/**
 * @brief Get a concrete value from the `Value` pointer.
 * @param value A `Value` pointer.
 * @param data_type The concrete data type you want to transform the value into.
 * @param pointer [Optional] A JSON pointer to a nested value.
 * @return A void pointer containing the concrete data type, or null if no data/wrong data type.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * enum DataType dt;
 * dt = String;
 * char *hostname = get_value(host->data, dt, "/hostname");
 *
 * if (!hostname) {
 *     printf("Could not find hostname in data!\n");
 *     exit(1);
 * }
 * @endcode
 */
extern void *get_value(void *value, enum DataType data_type, const char *pointer);

/**
 * @brief Returns the keys for an object-type `Value` pointer.
 * @param value A `Value` pointer.
 * @param pointer [Optional] A JSON pointer to a nested value.
 * @return An array of the `Value`'s keys, or null if no data or `Value` not an object.
 */
extern ValueKeysArray *get_value_keys(void *value, const char *pointer);

/**
 * @brief Returns the data type for a `Value` pointer.
 * @param value A `Value` pointer.
 * @param pointer [Optional] A JSON pointer to a nested value.
 * @return The `Value`'s data type, or null if no data.
 */
extern enum DataType *get_value_type(void *value, const char *pointer);

/**
 * @brief The shell command primitive for running commands on a
 * managed host.
 */
typedef struct _Command {
    const char *cmd; /**< The shell command */
} Command;

/**
 * @brief Result attributes returned from the managed host.
 */
typedef struct _CommandResult {
    int32_t exit_code; /**< Exit code for the shell command's process */
    char *stdout; /**< Process's standard output */
    char *stderr; /**< Process's standard error */
} CommandResult;

/**
 * @brief Create a new Command to represent your shell command.
 * @param cmd_str The shell command.
 * @return A new Command struct.
 *
 * #### Usage Example
 *
 * First, create a new Host struct to connect to your managed host:
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 * @endcode
 *
 * Then, create a new Command and send it to the managed host:
 *
 * @code
 * Command *command = command_new("whoami");
 * assert(command);
 * CommandResult *result = command_exec(command, host);
 * assert(result);
 *
 * printf("exit: %d, stdout: %s, stderr: %s", result->exit_code, result->stdout, result->stderr);
 * @endcode
 */
extern Command *command_new(const char *cmd_str);

/**
 * @brief Send request to the Agent to run your shell command.
 * @param cmd The command object.
 * @param host The host object you wish to run the command on.
 * @return A struct containing the execution results.
 */
extern CommandResult *command_exec(Command *cmd, Host *host);

/**
 * @brief Container for operating on a file.
 */
typedef struct _File {
    const char *path; /**< Absolute path to file on managed host */
} File;

/**
 * @brief Options for controlling file upload behaviour.
 */
typedef struct _FileOptions {
    const char *backup_existing; /**< Backup any existing file during upload using the provided suffix */
    uint64_t *chunk_size; /**< Size, in bytes, of each file chunk to be uploaded (default 1024b) */
} FileOptions;

/**
 * @brief Owner's user and group for a file.
 */
typedef struct _FileOwner {
    char *user_name; /**< User name */
    uint64_t user_uid; /**< User UID */
    char *group_name; /**< Group name */
    uint64_t group_gid; /**< Group GID */
} FileOwner;

/**
 * @brief Create a new File struct.
 * @param host The Host struct you wish to upload the file to.
 * @param path Absolute path to the file on your managed host.
 * @return A new File struct.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * File *file = file_new(host, "/path/to/file");
 * assert(file);
 * @endcode
 */
extern File *file_new(Host *host, const char *path);

/**
 * @brief Check if the file exists.
 * @param file The File struct you wish to check.
 * @param host The Host struct you wish to check the file on.
 * @return bool Whether the file exists.
 */
extern bool *file_exists(File *file, Host *host);

/**
 * @brief Upload a file to the managed host.
 * @param file The File struct you wish to upload.
 * @param host The Host struct you wish to upload to.
 * @param local_path Absolute path to the local file you wish to upload.
 * @param opts File options struct for controlling upload behaviour.
 * @return Return code - zero on success, non-zero on error.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * File *file = file_new(host, "/path/to/remote/file");
 * assert(file);
 * rc = file_upload(file, host, "/path/to/local/file", NULL);
 * assert(rc == 0);
 *
 * // Now let's upload another file and backup the original with the
 * // suffix '_bk'.
 * FileOptions opts;
 * strcpy(opts.backup_existing, "_bk");
 *
 * rc = file_upload(file, host, "/path/to/new/file", &opts);
 * assert(rc == 0);
 *
 * // Your remote path now has two entries:
 * // "/path/to/remote/file" and "/path/to/remote/file_bk"
 * @endcode
 */
extern int file_upload(File *file, Host *host, const char *local_path, FileOptions *opts);

/**
 * @brief Upload a file descriptor to the managed host.
 * @param file The File struct you wish to upload.
 * @param host The Host struct you wish to upload to.
 * @param file_descriptor C file descriptor for the file you wish to
 *        upload.
 * @param opts File options struct for controlling upload behaviour.
 * @return Return code - zero on success, non-zero on error.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * File *file = file_new(host, "/path/to/remote/file");
 * assert(file);
 * FILE *fp;
 * fp = fopen("/path/to/local/file", "r");
 * int fd = fileno(fp);
 * rc = file_upload_file(file, host, fd, NULL);
 * assert(rc == 0);
 * @endcode
 */
extern int file_upload_file(File *file, Host *host, int file_descriptor, FileOptions *opts);

/**
 * @brief Delete a file.
 * @param file The File struct you wish to delete.
 * @param host The Host struct you wish to delete a file on.
 * @return Return code - zero on success, non-zero on error.
 */
extern int file_delete(File *file, Host *host);

/**
 * @brief Move a file to a new path.
 * @param file The File struct you wish to move.
 * @param host The Host struct you wish to move a file on.
 * @param new_path The absolute file path you wish to move the file to.
 * @return Return code - zero on success, non-zero on error.
 */
extern int file_mv(File *file, Host *host, const char *new_path);

/**
 * @brief Copy a file to a new path.
 * @param file The File struct you wish to delete.
 * @param host The Host struct you wish to delete a file on.
 * @param new_path The absolute file path you wish to copy the file to.
 * @return Return code - zero on success, non-zero on error.
 */
extern int file_copy(File *file, Host *host, const char *new_path);

/**
 * @brief Get the file's owner.
 * @param file The File struct you wish to query.
 * @param host The Host struct you wish to query a file on.
 * @return FileOwner A FileOwner struct.
 */
extern FileOwner *file_get_owner(File *file, Host *host);

/**
 * @brief Set the file's owner.
 * @param file The File struct you wish to edit.
 * @param host The Host struct you wish to edit a file on.
 * @param user The user name of the new owner.
 * @param group The group name of the new owner.
 * @return Return code - zero on success, non-zero on error.
 */
extern int file_set_owner(File *file, Host *host, const char *user, const char *group);

/**
 * @brief Get the file's permissions mask.
 * @param file The File struct you wish to query.
 * @param host The Host struct you wish to query a file on.
 * @return uint16_t The permissions mask.
 */
extern uint16_t *file_get_mode(File *file, Host *host);

/**
 * @brief Set the file's permissions mask.
 * @param file The File struct you wish to edit.
 * @param host The Host struct you wish to edit a file on.
 * @param mode The new mask you wish to apply to the file.
 * @return Return code - zero on success, non-zero on error.
 */
extern int file_set_mode(File *file, Host *host, uint16_t mode);

/**
 * @brief Container for operating on a directory.
 */
typedef struct _Directory {
    const char *path; /**< Absolute path to dir on managed host */
} Directory;

/**
 * @brief Options for controlling directory operations.
 */
typedef struct _DirectoryOpts {
    bool do_recursive; /**< Perform action recursively */
} DirectoryOpts;

/**
 * @brief Create a new Directory struct.
 * @param host The Host struct you wish to manage a directory on.
 * @param path Absolute path to the directory on your managed host.
 * @return A new Directory struct.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * Directory *dir = directory_new(host, "/path/to/dir");
 * @endcode
 */
extern Directory *directory_new(Host *host, const char *path);

/**
 * @brief Check if the directory exists.
 * @param dir The Directory struct you wish to check.
 * @param host The Host struct you wish to check the directory on.
 * @return bool Whether the file exists.
 */
extern bool *directory_exists(Directory *dir, Host *host);

/**
 * @brief Create a directory.
 * @param dir The Directory struct you wish to create.
 * @param host The Host struct you wish to delete a directory on.
 * @param opts Directory options struct for controlling create
 *     behaviour.
 * @return Return code - zero on success, non-zero on error.
 */
extern int directory_create(Directory *dir, Host *host, DirectoryOpts *opts);

/**
 * @brief Delete a directory.
 * @param dir The Directory struct you wish to delete.
 * @param host The Host struct you wish to delete a directory on.
 * @param opts Directory options struct for controlling delete
 *     behaviour.
 * @return Return code - zero on success, non-zero on error.
 */
extern int directory_delete(Directory *dir, Host *host, DirectoryOpts *opts);

/**
 * @brief Move a directory to a new path.
 * @param dir The Directory struct you wish to move.
 * @param host The Host struct you wish to move a directory on.
 * @param new_path The absolute dir path you wish to move the dir to.
 * @return Return code - zero on success, non-zero on error.
 */
extern int directory_mv(Directory *dir, Host *host, char *new_path);

/**
 * @brief Get the directory's owner.
 * @param dir The Directory struct you wish to query.
 * @param host The Host struct you wish to query a dir on.
 * @return FileOwner A FileOwner struct.
 */
extern FileOwner *directory_get_owner(Directory *dir, Host *host);

/**
 * @brief Set the directory's owner.
 * @param dir The Directory struct you wish to edit.
 * @param host The Host struct you wish to edit a directory on.
 * @param user The user name of the new owner.
 * @param group The group name of the new owner.
 * @return Return code - zero on success, non-zero on error.
 */
extern int directory_set_owner(Directory *dir, Host *host, char *user, char *group);

/**
 * @brief Get the directory's permissions mask.
 * @param dir The Directory struct you wish to query.
 * @param host The Host struct you wish to query a directory on.
 * @return uint16_t The permissions mask.
 */
extern uint16_t *directory_get_mode(Directory *dir, Host *host);

/**
 * @brief Set the directory's permissions mask.
 * @param dir The Directory struct you wish to edit.
 * @param host The Host struct you wish to edit a directory on.
 * @param mode The new mask you wish to apply to the directory.
 * @return Return code - zero on success, non-zero on error.
 */
extern int directory_set_mode(Directory *dir, Host *host, uint16_t mode);

/**
 * @brief A list of supported Package providers.
 */
enum Providers {
    Default, /**< Automatically choose the default for a given platform */
    Apt,
    Dnf,
    Homebrew,
    Macports,
    Pkg,
    Ports,
    Yum,
};

/**
 * @brief The primitive for installing and manging software packages
 * on a managed host.
 */
typedef struct _Package {
    const char *name; /**< The name of the package, e.g. `nginx` */
    enum Providers provider; /**< The package source */
    bool installed; /**< Package installed bool */
} Package;

/**
 * @brief Create a new Package struct.
 * @param host The Host struct you want to install the package on.
 * @param name The name of the package, e.g. `nginx`.
 * @param providers The package provider you wish to target.
 * @return A new Package struct.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * enum Providers providers;
 * providers = Default;
 *
 * Package *package = package_new(host, "nginx", providers);
 * assert(package);
 * @endcode
 */
extern Package *package_new(Host *host, char *name, enum Providers providers);

/**
 * @brief Check if the package is installed.
 * @param package The Package struct.
 * @return Boolean indicating whether the package is currently installed (true) or not (false).
 *
 * #### Usage Example
 *
 * @code
 * Package *package = package_new(host, "nginx", "default");
 * assert(package);
 * bool *result = package_is_installed(package);
 * if (result) {
 *     printf("Package is installed!");
 * } else {
 *     printf("Package is not installed");
 * }
 * @endcode
 */
extern bool *package_is_installed(Package *package);

/**
 * @brief Install the package.
 * @param package The Package struct.
 * @param host The Host struct you wish to install the package on.
 * @return The CommandResult struct for the operation, or NULL if nothing was done.
 */
extern CommandResult *package_install(Package *package, Host *host);

/**
 * @brief Uninstall the package.
 * @param package The Package struct.
 * @param host The Host struct you wish to uninstall the package on.
 * @return The CommandResult struct for the operation, or NULL if nothing was done.
 */
extern CommandResult *package_uninstall(Package *package, Host *host);

/**
 * @brief Runnables are the executable items that a Service calls
 * actions on. Only one struct member (command OR service) should be
 * set for each Runnable.
 */
typedef struct _ServiceRunnable {
    char *command; /**< A script that is executed by the shell */
    char *service; /**< A daemon managed by the default system service manager */
} ServiceRunnable;

/**
 * @brief A mapping between an action (e.g. "start") and a Runnable.
 * To make this action the default action, use the name "_"
 * (underscore).
 */
typedef struct _ServiceAction {
    const char *action; /**< An instruction for the runnable, e.g. "start", "stop" etc. */
    ServiceRunnable runnable; /**< The Runnable for this action */
} ServiceAction;

/**
 * @brief Array of ServiceActions.
 */
typedef struct _ServiceActionArray {
    ServiceAction *ptr; /**< Actions */
    size_t length; /**< Size of array */
    size_t capacity; /**< Capacity of array */
} ServiceActionArray;

/**
 * @brief A mapping ('alias') between an action (e.g. "start") and
 * another action. For example, a mapped action could be used as an
 * alias for flags to pass to a Command Runnable, e.g. the alias
 * "start" could point to the action "-c /path/to/config.conf".
 */
typedef struct _ServiceMappedAction {
    const char *action; /**< The action alias */
    const char *mapped_action; /**< The action linked to a Runnable */
} ServiceMappedAction;

/**
 * @brief Array of ServiceMappedActions.
 */
typedef struct _ServiceMappedActionArray {
    ServiceMappedAction *ptr; /**< Actions */
    size_t length; /**< Size of array */
    size_t capacity; /**< Capacity of array */
} ServiceMappedActionArray;

/**
 * @brief The primitive for controlling services on a managed host.
 */
typedef struct _Service {
    ServiceActionArray actions; /**< Action Runnables */
    ServiceMappedActionArray *mapped_actions; /**< Action aliases */
} Service;

/**
 * @brief Create a new Service with a single Runnable.
 * @param runnable The default Runnable for this service.
 * @param mapped_actions An optional array of action aliases.
 * @param mapped_len Size of mapped array.
 * @return A new Service struct.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * ServiceRunnable runnable = { .service = "nginx" };
 *
 * ServiceMappedAction map_start = { .action = "start", .mapped_action = "-c /usr/local/etc/nginx.conf" };
 * ServiceMappedAction mapped[] = { map_start };
 *
 * Service *service = service_new_service(runnable, &mapped, sizeof mapped / sizeof *mapped);
 * assert(service);
 * @endcode
 */
extern Service *service_new_service(ServiceRunnable runnable, ServiceMappedAction *mapped_actions, size_t mapped_len);

/**
 * @brief Create a new Service with multiple Runnables.
 * @param actions An array of ServiceActions.
 * @param actions_len Size of actions array.
 * @param mapped_actions An optional array of action aliases.
 * @param mapped_len Size of mapped array.
 * @return A new Service struct.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * ServiceRunnable start_runnable = { .command = "/usr/local/bin/nginx" };
 * ServiceAction start_action = { .action = "start", .runnable = start_runnable };
 *
 * ServiceRunnable stop_runnable = { .command = "/usr/local/bin/nginx" };
 * ServiceAction stop_action = { .action = "stop", .runnable = stop_runnable };
 *
 * ServiceAction actions[] = { start_action, stop_action };
 *
 * Service *service = service_new_map(&actions, sizeof actions / sizeof *actions, NULL, 0);
 * assert(service);
 * @endcode
 */
extern Service *service_new_map(ServiceAction *actions, size_t actions_len, ServiceMappedAction *mapped_actions, size_t mapped_len);

/**
 * @brief Run a service action, e.g. "start" or "stop".
 * @param service The Service you wish to run the action on.
 * @param host The Host you wish to manage the service on.
 * @param action The action you wish to run.
 * @return A struct containing the execution results, or null if no action was required.
 */
extern CommandResult *service_action(Service *service, Host *host, char *action);

/**
 * @brief The primitive for opening and rendering templates.
 */
typedef struct _Template {
    void *inner; /**< Template internals */
} Template;

/**
 * @brief Template helper for building a hash data structure.
 */
typedef struct _MapBuilder {
    void *inner; /**< Internal data storage */
} MapBuilder;

/**
 * @brief Template helper for building a vector data structure.
 */
typedef struct _VecBuilder {
    void *inner; /**< Internal data storage */
} VecBuilder;

/**
 * @brief Create a new Template.
 * @param path File path to your template.
 * @return A new Template struct.
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_connect("nodes/mynode.json");
 * assert(host);
 *
 * Template *template = template_new("payloads/nginx/nginx.conf");
 * assert(template);
 *
 * MapBuilder *builder = map_new();
 * assert(builder);
 * int rc = map_insert_str(builder, "name", "Cyril Figgis");
 * assert(rc == 0);
 *
 * int fd = template_render_map(template, builder);
 * assert(rc != 0);
 *
 * File *file = file_new(host, "/usr/local/etc/nginx/nginx.conf");
 * assert(file);
 * rc = file_upload_file(file, host, fd, NULL);
 * assert(rc == 0);
 * @endcode
 */
extern Template *template_new(const char *path);

/**
 * @brief Render a Template using a MapBuilder data structure.
 * @param template The Template struct you want to render.
 * @param builder Data structure to pass to the template.
 * @return File descriptor - zero is error.
 */
extern int template_render_map(Template *template, MapBuilder *builder);

/**
 * @brief Render a Template using a VecBuilder data structure.
 * @param template The Template struct you want to render.
 * @param builder Data structure to pass to the template.
 * @return File descriptor - zero is error.
 */
extern int template_render_vec(Template *template, VecBuilder *builder);

/**
 * @brief Create a new MapBuilder instance that allows you to build
 *         a hash map data structure to pass to your template.
 * @return A new MapBuilder struct.
 */
extern MapBuilder *map_new();

/**
 * @brief Insert a string to the hash map.
 * @param builder The MapBuilder struct.
 * @param key The reference for the data item.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int map_insert_str(MapBuilder *builder, const char *key, const char *value);

/**
 * @brief Insert a boolean to the hash map.
 * @param builder The MapBuilder struct.
 * @param key The reference for the data item.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int map_insert_bool(MapBuilder *builder, const char *key, bool value);

/**
 * @brief Insert a vector (via VecBuilder) to the hash map.
 * @param builder The MapBuilder struct.
 * @param key The reference for the data item.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int map_insert_vec(MapBuilder *builder, const char *key, VecBuilder *value);

/**
 * @brief Insert a nested hash map (via MapBuilder) to the hash map.
 * @param builder The MapBuilder struct.
 * @param key The reference for the data item.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int map_insert_map(MapBuilder *builder, const char *key, MapBuilder *value);

/**
 * @brief Create a new VecBuilder instance that allows you to build
 *         a vector (array) data structure to pass to your template.
 * @return A new VecBuilder struct.
 */
extern VecBuilder *vec_new();

/**
 * @brief Insert a string to the vector.
 * @param builder The VecBuilder struct.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int vec_push_str(VecBuilder *builder, const char *value);

/**
 * @brief Insert a boolean to the vector.
 * @param builder The VecBuilder struct.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int vec_push_bool(VecBuilder *builder, bool value);

/**
 * @brief Insert a nested vector (via VecBuilder) to the vector.
 * @param builder The VecBuilder struct.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int vec_push_vec(VecBuilder *builder, VecBuilder *value);

/**
 * @brief Insert a hash map (via MapBuilder) to the vector.
 * @param builder The VecBuilder struct.
 * @param value The data item.
 * @return Return code - zero on success, non-zero on error.
 */
extern int vec_push_map(VecBuilder *builder, MapBuilder *value);

/**
 * @brief The payload's programming language.
 */
enum Language {
    C,
    Php,
    Rust,
};

/**
 * @brief Payloads are self-contained projects that encapsulate a
 * specific feature or system function. Think of them as reusable
 * chunks of code that can be run across multiple hosts. Any time you
 * have a task that you want to repeat, it should probably go into a
 * payload.
 */
typedef struct _Payload {
    const char *path; /**< Path to the payload directory */
    const char *artifact; /**< Name of the executable/source file to run */
    enum Language language; /**< Language the payload is written in */
} Payload;

/**
 * @brief Create a new Payload using the payload::artifact notation.
 * This notation is simply "payload" + separator ("::") +
 * "executable/source file". For example: "nginx::install".
 * @param payload_artifact The name of the payload and artifact in
 * payload::artifact notation.
 * @return A Payload struct, or null on error.
 */
extern Payload *payload_new(const char *payload_artifact);

/**
 * @brief Compile a payload's source code. This function is also
 * called by payload_run(), but is useful for precompiling payloads
 * ahead of time to catch build errors early.
 * @param payload The payload you wish to build.
 * @return Return code - zero on success, non-zero on error.
 */
extern int *payload_build(Payload *payload);

/**
 * @brief Execute the payload's artifact. For compiled languages, the
 * artifact will be executed directly. For interpreted languages, the
 * artifact will be passed as an argument to the interpreter.
 * @param payload The payload you wish to run.
 * @param host The host the payload will target.
 * @param user_args An optional array of args to pass to payload
 * executable.
 * @param user_args_len Size of user_args array.
 * @return Return code - zero on success, non-zero on error.
 */
extern int *payload_run(Payload *payload, Host *host, const char *user_args, size_t user_args_len);

#endif
