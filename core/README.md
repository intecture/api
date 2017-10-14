# Core API

The core API contains the endpoints used to configure a host, as well as the underlying OS abstractions that they are built on. Usually you'll use Intecture Proj ([proj/](../proj/)) instead, which reexports the core API, though for some applications, the core API will suffice.

## Project structure

The core API is organised into a series of directories for each endpoint (e.g. `command/`, `host/` etc.). Within each endpoint is a `providers/` directory that houses the underlying abstractions.

## Usage

For API usage, check out the [examples](examples/).
