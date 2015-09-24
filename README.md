# Introduction

Intecture is a developer friendly, multi-lingual configuration management tool for server systems.

Unlike Chef, Puppet et. al., Intecture has a few unique tricks:

* Extensible support for virtually any programming language
* Standard programming interface. No DSL. No magic.
    * Low level API (over ZMQ socket)
    * High level API (native Rust API with bindings)
* Built upon [ZeroMQ](http://zeromq.org) messaging library

You can find out more at [intecture.io](https://intecture.io).

# Install

First, as this project is written in Rust, you'll need...well, [Rust!](https://www.rust-lang.org)

Next, clone this repository to your local machine and use the Makefile to build it:

```
$ git clone #...
$ cd intecture-api/
$ make && sudo make install
```

Once this has finished, you should have a shiny new library called *libinapi.so*, which lives in your system's *lib/* directory.

# Uninstall

Run the uninstall target on the Makefile:

```
$ cd intecture-api/
$ sudo make uninstall
```

# Components

Intecture aims to be a fully distributed, service orientated system. Thus it is broken up into several modules.

## CLI

The CLI lives on developers' machines and is the primary interface with Intecture. Use it to kick off builds and manage the system.

## APIs

The APIs are the glue between your code and the systems you're managing. There are two variants:

### Low Level API

The low level API is exposed by the Agent (discussed below) as a stateless ZeroMQ socket endpoint. While you can consume this API directly, it's recommended that you use the high level API, which gives you substantially more functionality.

### High Level API

The high level API is a wrapper for the low level socket API, written in Rust. This API gives users a more stateful interface to their systems with features like callbacks and data abstractions (currently JSON only).

While this API is written in Rust, there are several bindings to connect your language with the Rust API. We're currently aiming to support PHP and Ruby as our pilot languages.

## Agent

Each server that you want to manage must run the Intecture Agent. This service exposes a low level API that wraps basic system functions ("primitives"), such as filesystem operations, process management etc. The agent will translate calls to this API into native operations on the server, returning the result to the caller.

# Support

For enterprise support and consulting services, please email <mailto:support@intecture.io>.

For any bugs, feature requests etc., please ticket them on GitHub.