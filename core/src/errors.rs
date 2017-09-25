// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

error_chain! {
    errors {
        InvalidSysctlKey(k: String) {
            description("Provided key not found in sysctl output"),
            display("Provided key '{}' not found in sysctl output", k),
        }

        ProviderUnavailable(p: &'static str) {
            description("No providers available"),
            display("No providers available for {}", p),
        }

        RemoteProvider {
            endpoint: &'static str,
            func: &'static str
        } {
            description("Could not run provider function on host"),
            display("Could not run {}::{}() on host", endpoint, func),
        }

        SystemCommand(c: &'static str) {
            description("Error running system command"),
            display("Error running system command '{}'", c),
        }

        SystemCommandOutput(c: &'static str) {
            description("Could not understand output of system command (not UTF-8 compliant)"),
            display("Could not understand output of system command '{}' (not UTF-8 compliant)", c),
        }
    }
}
