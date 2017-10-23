# 0.4.0 (in progress)

Version 0.4 is a complete rewrite of Intecture's core API. If we had been running a stable release (i.e. 1.0), this version would be version 2.0, but sadly we have to go with the far less dramatic 0.4 :(

Anyway, there are some big headlines in this release:

- **Hello Tokio!** This is an exciting change to our socket API, as we bid a fond farewell to ZeroMQ. The change to Tokio will bring loads of benefits, such as substantially reducing our non-Rust dependencies, and introducing strongly typed messaging between servers and clients. This will make our communications more robust and less open to exploitation. Woo hoo!
- **Asynchronous, streaming endpoints with `Futures`!** This is another huge change to the API, which allows configuration tasks to be run in parallel; moreover it allows users to stream task output in realtime. Gone are the days of waiting in the dark for tasks to complete!
- **Greater emphasis on composability.** Each endpoint (formerly called _primitives_) is organised into a collection of _providers_ that will (you guessed it) provide target-specific implementations of the endpoint. Users will be able to select an endpoint provider manually or let the system choose the best provider for the target platform.
- **Separation of duties.** In the previous versions of Intecture, the API had been organised into a single project, making it cluttered and unwieldy. For the next release, things like the FFI, language bindings and project boilerplate will be moved into separate child projects under the same Cargo workspace.
