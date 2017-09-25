# Intecture API

This is a major restructuring of Intecture's core components. It goes like this:

- [core](core/) - The core API that does all the heavy lifting
- [bindings](bindings/) - Rust FFI and language bindings
- [proj](proj/) - Project helper functions
- [agent](agent/) - Tiny daemon that wraps the core API
