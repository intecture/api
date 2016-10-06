<?php
/**
 * Intecture API - PHP binding stubs
 *
 * @author Between Lines <info@betweenlines.co.uk>
 * @version 0.2.1
 */

namespace Intecture;

/**
 * The shell command primitive for running commands on a managed
 * host.
 *
 * @example stub_examples.php 7 17 Basic usage
 * @example stub_examples.php 29 12 Reuse across multiple hosts
 */
class Command
{
    /**
     * Create a new Command to represent your shell command.
     *
     * @param string $cmd The shell command.
     */
    public function __construct($cmd) {}

    /**
     * Send request to the Agent to run your shell command.
     *
     * @param Host $host The Host object connected to the managed
     *     host you want to run the command on.
     *
     * @throws CommandException if $host is not an instance of Host.
     *
     * @return array Result attributes returned from the managed
     *     host.
     */
    public function exec($host) {}
}

/**
 * The exception class for Command errors.
 */
class CommandException {}

/**
 * The Directory primitive for managing dirs on a managed host.
 *
 * @example stub_examples.php 46 11 Basic usage
 */
class Directory
{
    /**
     * Perform an action recursively.
     */
    const OPT_DO_RECURSIVE = 31;

    /**
     * Create a new Directory object to manage a dir on a remote host.
     *
     * @param Host $host The Host object you want to manage the dir
     *     on.
     * @param string $path Absolute path to the dir on your managed
     *     host.
     */
    public function __construct($host, $path) {}

    /**
     * Check if the directory exists.
     *
     * @param Host $host The Host object you want to manage the dir
     *     on.
     *
     * @return bool Whether the directory exists.
     */
    public function exists($host) {}

    /**
     * Create the directory.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     * @param array $options Optional parameters that tweak the way
     *     directory deletion occurs.
     */
    public function create($host, $options = array()) {}

    /**
     * Delete the directory.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     * @param array $options Optional parameters that tweak the way
     *     directory deletion occurs.
     */
    public function delete($host, $options = array()) {}

    /**
     * Move the directory to a new path.
     *
     * @param Host $host The Host object you want to manage the dir
     *     on.
     * @param string $new_path The absolute path you wish to move the
     *     dir to.
     */
    public function mv($host, $new_path) {}

    /**
     * Get the directory's owner.
     *
     * @param Host $host The Host object you want to manage the dir
     *     on.
     *
     * @return array An array containing the owner and group names
     *     and IDs.
     */
    public function get_owner($host) {}

    /**
     * Set the directory's owner.
     *
     * @param Host $host The Host object you want to manage the dir
     *     on.
     * @param string $user The user name of the new owner.
     * @param string $group The group name of the new owner.
     */
    public function set_owner($host, $user, $group) {}

    /**
     * Get the directory's permissions mask.
     *
     * @param Host $host The Host object you want to manage the dir
     *     on.
     *
     * @return int The permissions mask.
     */
    public function get_mode($host) {}

    /**
     * Set the directory's permissions mask.
     *
     * @param Host $host The Host object you want to manage the dir
     *     on.
     * @param int $mode The new mask you wish to apply to the dir.
     */
    public function set_mode($host, $mode) {}
}

/**
 * The exception class for Directory errors.
 */
class DirectoryException {}

/**
 * The File primitive for managing files on a managed host.
 *
 * @example stub_examples.php 62 11 Basic usage
 * @example stub_examples.php 78 9 Uploading a file
 */
class File
{
    /**
     * Backup existing file during file upload, rather than
     * overwriting it.
     */
    const OPT_BACKUP_EXISTING = 11;

    /**
     * Controls the size, in bytes, of each file chunk to be uploaded
     * to the Agent. Defaults to 1024.
     */
    const OPT_CHUNK_SIZE = 12;

    /**
     * Create a new File object to manage a file on a remote host.
     *
     * @param Host $host The Host object you want to manage the file
     *     on.
     * @param string $path Absolute path to the file on your managed
     *     host.
     */
    public function __construct($host, $path) {}

    /**
     * Check if the file exists.
     *
     * @param Host $host The Host object you want to manage the file
     *     on.
     *
     * @return bool Whether the file exists.
     */
    public function exists($host) {}

    /**
     * Upload a file to the managed host.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     * @param string $local_path Path to the local file you wish to
     *     upload.
     * @param array $options Optional parameters that tweak the
     *     uploader's behaviour.
     */
    public function upload($host, $local_path, $options = array()) {}

    /**
     * Delete the file.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     */
    public function delete($host) {}

    /**
     * Move the file to a new path.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     * @param string $new_path The absolute file path you wish to
     *     move the file to.
     */
    public function mv($host, $new_path) {}

    /**
     * Copy the file to a new path.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     * @param string $new_path The absolute file path you wish to
     *     copy the file to.
     */
    public function copy($host, $new_path) {}

    /**
     * Get the file's owner.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     *
     * @return array An array containing the owner and group names
     *     and IDs.
     */
    public function get_owner($host) {}

    /**
     * Set the file's owner.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     * @param string $user The user name of the new owner.
     * @param string $group The group name of the new owner.
     */
    public function set_owner($host, $user, $group) {}

    /**
     * Get the file's permissions mask.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     *
     * @return int The permissions mask.
     */
    public function get_mode($host) {}

    /**
     * Set the file's permissions mask.
     *
     * @param Host $host The Host object you want to manage the files
     *     on.
     * @param int $mode The new mask you wish to apply to the file.
     */
    public function set_mode($host, $mode) {}
}

/**
 * The exception class for File errors.
 */
class FileException {}

/**
 * The host wrapper for communicating with a managed host.
 *
 * The Host object will automatically connect to your host and will
 * maintain the connection until it is destroyed.
 *
 * @example stub_examples.php 92 3 Basic usage
 */
class Host
{
    /**
     * Create a new Host to communicate with a managed host.
     */
    public function __construct() {}

    /**
     * Connect Host to a managed host.
     *
     * @param string $hostname The hostname or IP address of your managed
     *     host.
     * @param int $api_port The port number of the Agent API service on
     *     $hostname.
     * @param int $upload_port The port number of the Agent file transfer
     *     service on $hostname.
     */
    public function connect($hostname, $api_port, $upload_port) {}
}

/**
 * The exception class for Host errors.
 */
class HostException {}

/**
 * The wrapper for installing and managing software packages on a
 * managed host.
 *
 * @example stub_examples.php 100 11 Basic usage
 * @example stub_examples.php 116 3 Specify provider
 */
class Package
{
    /**
     * Use the Aptitude provider.
     */
    const PROVIDER_APT = 1;

    /**
     * Use the DNF provider.
     */
    const PROVIDER_DNF = 2;

    /**
     * Use the Homebrew provider.
     */
    const PROVIDER_HOMEBREW = 3;

    /**
     * Use the Macports provider.
     */
    const PROVIDER_MACPORTS = 4;

    /**
     * Use the Pkg provider.
     */
    const PROVIDER_PKG = 5;

    /**
     * Use the Ports provider.
     */
    const PROVIDER_PORTS = 6;

    /**
     * Use the Yum provider.
     */
    const PROVIDER_YUM = 7;

    /**
     * Create a new Package.
     *
     * If you have multiple package providers, you can specify one
     * or allow Intecture to select a default based on the OS.
     *
     * @param Host $host The Host object connected to the managed
     *     host you want to install the package on.
     * @param string $name The name of the package, e.g. `nginx`.
     * @param constant $provider The package provider that will be
     *     used to install the package. If NULL, the default will be
     *     used for the provided host.
     */
    public function __construct($host, $name, $provider = NULL) {}

    /**
     * Check if the package is installed.
     *
     * @return bool Whether the package is installed.
     */
    public function is_installed() {}

    /**
     * Install the package.
     *
     * @param Host $host The Host object connected to the managed
     *     host you wish to install the package on.
     *
     * @return array|null Result attributes returned from the managed
     *     host, or null if nothing had to be done.
     */
    public function install($host) {}

    /**
     * Uninstall the package.
     *
     * @param Host $host The Host object connected to the managed
     *     host you wish to uninstall the package on.
     *
     * @return array|null Result attributes returned from the managed
     *     host, or null if nothing had to be done.
     */
    public function uninstall($host) {}
}

/**
 * The exception class for Package errors.
 */
class PackageException {}

/**
 * The wrapper for managing services on a managed host.
 *
 * @example stub_examples.php 125 13 Basic Service usage
 * @example stub_examples.php 143 12 Basic Command usage
 * @example stub_examples.php 160 10 Multiple Runnables
 * @example stub_examples.php 175 3 Mapped actions
 */
class Service
{
    /**
     * Create a new Service.
     *
     * @param mixed $actions Can be: (1) a ServiceRunnable object
     *     which is used as the default Runnable for all actions, or
     *     (2) an associative array of ServiceRunnables indexed by
     *     action name. To specify multiple Runnables with a default,
     *     use the index "_" (underscore) to identify your default
     *     action.
     * @param array $mapped_actions A mapping ('alias') between an
     *     action (e.g. "start") and another action. For example, a
     *     mapped action could be used as an alias for flags to pass
     *     to a Command Runnable, e.g. the alias "start" could point
     *     to the action "-c /path/to/config.conf".
     */
    public function __construct($actions, $mapped_actions = NULL) {}

    /**
     * Run a service action, e.g. "start" or "stop".
     *
     * @param Host $host The Host object you wish to run a service
     *     action on.
     * @param string $action The action you wish run against the
     *     Service.
     *
     * @return array Result attributes returned from the managed
     *     host, or null if no action was required.
     */
    public function action($host, $action) {}
}

/**
 * Runnables are the executable items that a Service calls actions
 * on.
 *
 * @example stub_examples.php 183 5 Basic usage
 */
class ServiceRunnable
{
    /**
     * A script that is executed by the shell.
     */
    const COMMAND = 21;

    /**
     * A daemon managed by the default system service manager.
     */
    const SERVICE = 22;

    /**
     * Create a new ServiceRunnable.
     *
     * @param string $runnable A shell command or the name of the
     *     service.
     * @param constant $type The type of Runnable - a shell command
     *     or a service.
     */
    public function __construct($runnable, $type) {}
}

/**
 * The exception class for Service errors.
 */
class ServiceException {}

/**
 * Data structures containing information about your managed host.
 *
 * The Telemetry object stores metadata about a host, such as its
 * network interfaces, disk mounts, CPU stats and hostname.
 *
 * @example stub_examples.php 194 14 Basic usage
 * @example stub_examples.php 213 7 Load data for multiple hosts
 */
class Telemetry
{
    /**
     * Load the telemetry data for a managed host.
     *
     * @param Host $host The Host object connected to the managed
     *     host you want to run the command on.
     *
     * @throws TelemetryException if $host is not an instance of Host.
     *
     * @return Telemetry An object holding the telemetry data for
     *     your managed host.
     */
    public static function load($host) {}

    /**
     * Retrieve the telemetry data from the object.
     *
     * @return array An array containing all the telemetry data
     *     returned by the managed host.
     */
    public function get() {}
}

/**
 * The exception class for Telemetry errors.
 */
class TelemetryException {}

/**
 * The Template primitive for opening and rendering templates.
 */
class Template {
    /**
     * Create a new Template.
     *
     * @param string $path File path to your template.
     */
    public function __construct($path) {}

    /**
     * Render a Template using a Map/VecBuilder data structure.
     *
     * @param mixed $builder MapBuilder or VecBuilder object.
     */
    public function render($builder) {}
}

/**
 * The exception class for Template errors.
 */
class TemplateException {}

/**
 */
class MapBuilder {

}

/**
 */
class VecBuilder {

}
