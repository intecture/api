# Intecture Agent

Intecture Agent is a tiny daemon that exposes Intecture's API as a service. To consume this service, use the `host::remote::Plain` type from [core](../core/).

## Usage

To run the agent, simply execute the `intecture_agent` binary, remembering to pass it a socket address to listen on.

For example, to listen to localhost on port 7101, run:

```sh
intecture_agent --address localhost:7101
```

More likely though you'll want to listen on your public interface so that Intecture can talk to the host remotely. In this case you should specify the host's IP address manually, or use `0.0.0.0` to listen on all interfaces.

## Config file

You can also store agent parameters in a configuration file. The file must be in TOML format, and can live anywhere on your server. It should look like this:

```toml
address = "0.0.0.0:7101"
```

Once you've created a config file, pass it to the agent binary:

```sh
intecture_agent --config agent.toml
```
