// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

#[cfg(feature = "local-run")]
pub mod bin_resolver;

#[cfg(all(target_os = "linux", feature = "local-run"))]
#[allow(dead_code)]
pub mod debian_base;

#[cfg(feature = "local-run")]
#[allow(dead_code)]
pub mod default_base;

#[cfg(all(target_os = "linux", feature = "local-run"))]
#[allow(dead_code)]
pub mod linux_base;

#[cfg(all(target_os = "linux", feature = "local-run"))]
#[allow(dead_code)]
pub mod redhat_base;

#[cfg(all(any(target_os = "freebsd", target_os = "macos"), feature = "local-run"))]
#[allow(dead_code)]
pub mod unix_base;

#[cfg(all(target_os = "linux", feature = "local-run"))]
pub mod debian;

#[cfg(all(target_os = "linux", feature = "local-run"))]
pub mod centos;

#[cfg(all(target_os = "linux", feature = "local-run"))]
pub mod fedora;

#[cfg(all(target_os = "freebsd", feature = "local-run"))]
pub mod freebsd;

#[cfg(all(target_os = "linux", feature = "local-run"))]
#[allow(dead_code)]
pub mod linux;

#[cfg(all(target_os = "macos", feature = "local-run"))]
pub mod macos;

#[cfg(all(target_os = "linux", feature = "local-run"))]
pub mod redhat;

#[cfg(all(target_os = "linux", feature = "local-run"))]
pub mod ubuntu;

#[cfg(feature = "remote-run")]
pub mod remote;

pub struct Target;
