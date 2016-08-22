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
} Host;

/**
 * @brief Create a new Host to represent your managed host.
 * @return A new Host struct.
 *
 * #### Usage Example
 *
 * Create a new Host struct to connect to your managed host:
 *
 * @code
 * Host *host = host_new();
 * assert(host);
 * @endcode
 */
extern Host *host_new();

/**
 * @brief Connects a Host to a remote agent.
 * @param host The Host struct you wish to connect.
 * @param hostname The IP address or hostname of your managed host.
 * @param api_port The port number that the Agent API service is listening on.
 * @param upload_port The port number that the Agent File Upload service is listening on.
 * @param auth_server The ip address/hostname and port number of your auth server.
 *
 * #### Usage Example
 *
 * Connect your Host struct to your managed host.
 *
 * @code
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
 * @endcode
 */
extern uint8_t host_connect(Host *host, const char *hostname, uint32_t api_port, uint32_t upload_port, const char *auth_server);

/**
 * @brief Close the connection to your managed host.
 * @param host The host connection you wish to close.
 */
extern uint8_t host_close(Host *host);

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
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
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
 * @brief CPU information for the telemetry primitive.
 */
typedef struct _Cpu {
    char *vendor; /**< CPU vendor */
    char *brand_string; /**< Full description of CPU */
    uint32_t cores; /**< Total number of cores */
} Cpu;

/**
 * @brief File system information for the telemetry primitive.
 */
typedef struct _FsMount {
    char *filesystem; /**< File system being mounted */
    char *mountpoint; /**< Location of mount */
    uint64_t size; /**< Size on disk (in kb) */
    uint64_t used; /**< Disk space used (in kb) */
    uint64_t available; /**< Disk space available (in kb) */
    float capacity; /**< Percentage capacity available */
//    uint64_t inodes_used; /**< Inodes used */
//    uint64_t inodes_available; /**< Inodes available */
//    float inodes_capacity; /**< Percentage capacity available */
} FsMount;

/**
 * @brief Array of file systems for the telemetry primitive.
 */
typedef struct _FsArray {
    FsMount *ptr; /**< File system mounts */
    size_t length; /**< Size of array */
    size_t capacity; /**< Capacity of array */
} FsArray;

/**
 * @brief IPv4 address information for the network interface.
 */
typedef struct _NetifIPv4 {
    char *address; /**< IPv4 address */
    char *netmask; /**< Netmask */
} NetifIPv4;

/**
 * @brief IPv6 address information for the network interface.
 */
typedef struct _NetifIPv6 {
    char *address; /**< IPv6 address */
    uint32_t prefixlen; /**< Prefix length */
    char *scopeid; /**< Scope ID */
} NetifIPv6;

/**
 * @brief Network interface information for the telemetry primitive.
 */
typedef struct _Netif {
    char *interface; /**< Name of the interface */
    char *mac; /**< (Optional) MAC address */
    NetifIPv4 *inet; /**< (Optional) IPv4 address */
    NetifIPv6 *inet6; /**< (Optional) IPv6 address */
    char *status; /**< (Optional) Interface status: Active|Inactive */
} Netif;

/**
 * @brief Array of network interfaces for the telemetry primitive.
 */
typedef struct _NetifArray {
    Netif *ptr; /**< File system mounts */
    size_t length; /**< Size of array */
    size_t capacity; /**< Capacity of array */
} NetifArray;

/**
 * @brief Operating system information for the telemetry primitive.
 */
typedef struct _Os {
    char *arch; /**< OS architecture (e.g. x86_64) */
    char *family; /**< OS family (e.g. unix) */
    char *platform; /**< OS platform (e.g. freebsd) */
    char *version; /**< 10.1 */
} Os;

/**
 * @brief The telemetry primitive for gathering system information on
 * a managed host.
 */
typedef struct _Telemetry {
    Cpu cpu; /**< CPU info */
    FsArray fs; /**< Array of file system mounts */
    char *hostname; /**< Hostname of the machine */
    uint64_t memory; /**< Total memory (in kb) */
    NetifArray net; /**< Network interfaces */
    Os os; /**< Operating system info */
} Telemetry;

/**
 * @brief Create a new Telemetry to hold information about your
 * managed host.
 * @param host The Host struct you wish to gather telemetry on.
 * @return A new Telemetry struct.
 *
 * #### Usage Example
 *
 * Initialize a new Telemetry struct to connect to your managed host:
 *
 * @code
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
 *
 * Telemetry *telemetry = telemetry_init(host);
 * assert(telemetry);
 * @endcode
 */
extern Telemetry *telemetry_init(Host *host);

/**
 * @brief Destroy the Telemetry struct and free its memory.
 * @param telemetry The telemetry object you wish to destroy.
 *
 * #### Warning!
 *
 * Do not attempt to free the Telemetry struct using free(), as
 * memory is allocated idiomatically by Rust and is incompatible with
 * C free().
 */
extern uint8_t telemetry_free(Telemetry *telemetry);

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
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
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
 *
 * #### Usage Example
 *
 * @code
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
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
extern uint8_t file_upload(File *file, Host *host, const char *local_path, FileOptions *opts);

/**
 * @brief Delete a file.
 * @param file The File struct you wish to delete.
 * @param host The Host struct you wish to delete a file on.
 */
extern uint8_t file_delete(File *file, Host *host);

/**
 * @brief Move a file to a new path.
 * @param file The File struct you wish to move.
 * @param host The Host struct you wish to move a file on.
 * @param new_path The absolute file path you wish to move the file to.
 */
extern uint8_t file_mv(File *file, Host *host, const char *new_path);

/**
 * @brief Copy a file to a new path.
 * @param file The File struct you wish to delete.
 * @param host The Host struct you wish to delete a file on.
 * @param new_path The absolute file path you wish to copy the file to.
 */
extern uint8_t file_copy(File *file, Host *host, const char *new_path);

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
 */
extern uint8_t file_set_owner(File *file, Host *host, const char *user, const char *group);

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
 */
extern uint8_t file_set_mode(File *file, Host *host, uint16_t mode);

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
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
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
 */
extern uint8_t directory_create(Directory *dir, Host *host, DirectoryOpts *opts);

/**
 * @brief Delete a directory.
 * @param dir The Directory struct you wish to delete.
 * @param host The Host struct you wish to delete a directory on.
 * @param opts Directory options struct for controlling delete
 *     behaviour.
 */
extern uint8_t directory_delete(Directory *dir, Host *host, DirectoryOpts *opts);

/**
 * @brief Move a directory to a new path.
 * @param dir The Directory struct you wish to move.
 * @param host The Host struct you wish to move a directory on.
 * @param new_path The absolute dir path you wish to move the dir to.
 */
extern uint8_t directory_mv(Directory *dir, Host *host, char *new_path);

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
 */
extern uint8_t directory_set_owner(Directory *dir, Host *host, char *user, char *group);

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
 */
extern uint8_t directory_set_mode(Directory *dir, Host *host, uint16_t mode);

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
 * @brief Result of package operation.
 */
enum PackageResult {
    Result, /**< The command result from a package operation (e.g. installing/uninstalling) */
    NoAction /**< No action was necessary to achieve the desired state (e.g. calling install() on a currently installed package) */
};

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
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
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
 * @param result The CommandResult struct for the operation will be bound to this argument.
 * @return An enum indicating whether any command was run or not.
 */
extern enum PackageResult *package_install(Package *package, Host *host, CommandResult *result);

/**
 * @brief Uninstall the package.
 * @param package The Package struct.
 * @param host The Host struct you wish to uninstall the package on.
 * @param result The CommandResult struct for the operation will be bound to this argument.
 * @return An enum indicating whether any command was run or not.
 */
extern enum PackageResult *package_uninstall(Package *package, Host *host, CommandResult *result);

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
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
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
 * Host *host = host_new();
 * assert(host);
 * int rc = host->connect("example.com", 7101, 7102, "auth.example.com:7101");
 * assert(rc == 0);
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

#endif
