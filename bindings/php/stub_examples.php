<?php

/*
 * Command
 * Basic usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

// Create a new command and send it to the managed host
$cmd = new Intecture\Command('whoami');
$result = $cmd->exec($host);
print_r($result);

Output:

Array
(
    [exit_code] => 0 // Exit code for the shell command's process
    [stdout] => root // Process's standard output
    [stderr] =>      // Process's standard error
)

/*
 * Command
 * Reuse across multiple hosts
 */
// Web server
$web = new Host();
$web->connect('web.example.com', 7101, 7102, 'auth.example.com:7101');

// Database server
$db = new Host();
$db->connect('db.example.com', 7101, 7102, 'auth.example.com:7101');

$cmd = new Intecture\Command('whoami');

$web_result = $cmd->exec($web);
$db_result = $cmd->exec($db);

/*
 * Directory
 * Basic usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

// Create a new Directory object to manage a specific directory
$dir = new Directory($host, '/path/to/dir');
if ($dir->exists($host)) {
    echo 'This directory exists';
} else {
    echo 'This directory doesn\'t exist';
}

/*
 * File
 * Basic usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

// Create a new File object to manage a specific file
$file = new File($host, '/path/to/file');
if ($file->exists($host)) {
    echo 'This file exists';
} else {
    echo 'This file doesn\'t exist';
}

/*
 * File
 * Uploading a file
 */
$file = new File($host, '/path/to/file');
// Upload my_local_file.txt to /path/to/file
// If it already exists on the host, back it up using _bk as the
// suffix.
// For example, /path/to/file would become /path/to/file_bk and the
// new uploaded file would reside at /path/to/file.
$file->upload($host, 'my_local_file.txt', array(
    File::OPT_BACKUP_EXISTING => '_bk'
));

/*
 * Host
 * Basic usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

/*
 * Package
 * Basic usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

// Create Package object to install a package from the default source
$package = new Package($host, 'nginx');
$result = $package->install($host);

if ($result['exit_code'] != 0) {
    throw new Exception('Eep! I couldn\'t install nginx');
}

/*
 * Package
 * Specify provider
 */
// Alternatively, you can specify a provider if you already know the
// Host's OS
$package = new Package($host, 'nginx', Package::PROVIDER_MACPORTS);


/*
 * Service
 * Basic Service usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

// Setup a new Service with a Service type Runnable
$runnable = new ServiceRunnable('nginx', ServiceRunnable::SERVICE);
$service = new Service($runnable);

$result = $service->action($host, 'start');

if ($result['exit_code'] != 0) {
    throw new Exception('Eep! I couldn\'t start nginx');
}

/*
 * Service
 * Basic Command usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

// Setup a new Service with a Command type Runnable
$runnable = new ServiceRunnable('/usr/local/bin/nginx', ServiceRunnable::COMMAND);
$service = new Service($runnable);

$result = $service->action($host, 'enable');
assert($result['exit_code'] === 0);
$result = $service->action($host, 'start');
assert($result['exit_code'] === 0);

/*
 * Service
 * Multiple Runnables
 */
$runnables = array(
    "start" => new ServiceRunnable('/usr/local/bin/start_mysvc', ServiceRunnable::COMMAND),
    "stop" => new ServiceRunnable('/usr/local/bin/stop_mysvc', ServiceRunnable::COMMAND),
    "_" => new ServiceRunnable('curl -s http://localhost/service-status | grep -i', ServiceRunnable::COMMAND)
);
$service = new Service($runnables);

$service->action($host, 'start');
$result = $service->action($host, 'requests_per_sec'); // Runs command "curl -s http://localhost/service-status | grep -i requests_per_sec"
echo $result['stdout'];

/*
 * Service
 * Mapped actions
 */
$runnable = new ServiceRunnable('nginx', ServiceRunnable::SERVICE);
$service = new Service($runnable, array('start' => 'load'));
$service->action($host, 'start'); // Maps to action "load"

/*
 * ServiceRunnable
 * Basic usage
 */
// Service type Runnable
$runnable = new ServiceRunnable('/usr/local/bin/nginx', ServiceRunnable::COMMAND);

// Or a Service type Runnable
$runnable = new ServiceRunnable('nginx', ServiceRunnable::SERVICE);


/*
 * Telemetry
 * Basic usage
 */
// Setup Host object to communicate with managed host
$host = new Host();
$host->connect('example.com', 7101, 7102, 'auth.example.com:7101');

// Load a new Telemetry object to hold your host's telemetry data
$telemetry = Telemetry::load($host);
print_r($telemetry->get());

Output:

Array
(
    ...lots of data about a host
)

/*
 * Telemetry
 * Load data for multiple hosts
 */
$web = new Host();
$web->connect('web.example.com', 7101, 7102, 'auth.example.com:7101');
$web_data = Telemetry::load($web);

$db = new Host();
$db->connect('db.example.com', 7101, 7102, 'auth.example.com:7101');
$db_data = Telemetry::load($db);
