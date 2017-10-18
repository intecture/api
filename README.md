# Intecture APIs [![Build Status](https://travis-ci.org/intecture/api.svg?branch=master)](https://travis-ci.org/intecture/api) [![Coverage Status](https://coveralls.io/repos/github/Intecture/api/badge.svg?branch=master)](https://coveralls.io/github/Intecture/api?branch=master) [![Gitter](https://badges.gitter.im/Join\%20Chat.svg)](https://gitter.im/intecture/Lobby)

**Intecture is an API for managing your servers. Visit [intecture.io](http://intecture.io).**

**API docs can be found here: [intecture.io/api/intecture_api/](http://intecture.io/api/intecture_api/).**

---

Intecture's APIs (_cough_ and a binary) are the heart and soul of Intecture. Check out each component's `README.md` for details:

- [core](core/) - The core API that does all the heavy lifting
- [bindings](bindings/) - Rust FFI and language bindings
- [proj](proj/) - Helpers and boilerplate for building Intecture projects
- [agent](agent/) - Tiny daemon that exposes the core API as a service (for your hosts!)

## Getting started

Intecture is pretty light on external dependencies. In fact, all you'll need to [get started is Rust](https://www.rust-lang.org/install.html)!

Once you've installed Rust, create a new Cargo binary project:

```
cargo new --bin
```

Next, add Intecture to your `Cargo.toml`:

```
[dependencies]
futures = "0.1"
intecture_api = {git = "https://github.com/intecture/api", version = "0.4"}
tokio-core = "0.1"
```

This is all we need to do if we only want to manage the local machine. Just make sure you use the `Local` host type, like so:

```rust
let host = Local::new().and_then(|host| {
    // Do stuff on the local host...
});
```

You can find some basic examples here: [core/examples](core/examples).
Also you should refer to the API docs: [intecture.io/api/intecture_api/](http://intecture.io/api/intecture_api/)

#### For remote hosts only

To manage a remote machine with Intecture, you need to take a few extra steps. On the remote machine...

[Install Rust](https://www.rust-lang.org/install.html).

Clone this GitHub repository:

```
git clone https://github.com/intecture/api
```

Build the project:

```
cd api && cargo build --release
```

Then run the agent, specifying an address to bind to:

```
target/release/intecture_agent --address 0.0.0.0:7101
```

Remember, the agent _must_ be running in order for the API to connect to this host.

Finally we can get back to what we came here for - Rust codez! To manage this machine, make sure you use the `Plain` remote host type, like so:

```rust
let host = Plain::connect("<ip_of_host>:7101").and_then(|host| {
    // Do stuff on the local host...
});
```

Note the type is `Plain`, rather than `Remote`. At the moment, **Intecture does no encryption**, making it unsafe for use over insecure networks (i.e. the internet). The type `Plain` signifies this. In the future we will add support for encrypted remote host types as well, but for now, we cannot recommend strongly enough that you only use this on a secure local network.

## What's new?

Check out [RELEASES.md](RELEASES.md) for details.
