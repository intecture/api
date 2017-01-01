// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

fn main() {
    let local = cfg!(feature = "local-run");
    let remote = cfg!(feature = "remote-run");

    if local && remote {
        panic!("Mutually exclusive features `local-run` and `remote-run`. You must only enable one.");
    }
    else if !local && !remote {
        panic!("Missing feature `local-run` or `remote-run`. You must enable one.");
    }
}
