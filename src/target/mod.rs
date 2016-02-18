// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

#[cfg(feature = "local-run")]
pub mod bin_resolver;

#[cfg(feature = "local-run")]
#[allow(dead_code)]
pub mod default_base;

#[cfg(feature = "remote-run")]
pub mod remote;

#[cfg(all(target_os = "linux", feature = "local-run"))]
#[allow(dead_code)]
pub mod linux_base;

#[cfg(all(in_os_family = "redhat", feature = "local-run"))]
#[allow(dead_code)]
pub mod redhat_base;

#[cfg(all(in_os_family = "unix", feature = "local-run"))]
#[allow(dead_code)]
pub mod unix_base;

#[cfg(all(in_os_platform = "centos", feature = "local-run"))]
pub mod centos;

#[cfg(all(in_os_platform = "debian", feature = "local-run"))]
pub mod debian;

#[cfg(all(in_os_platform = "fedora", feature = "local-run"))]
pub mod fedora;

#[cfg(all(in_os_platform = "freebsd", feature = "local-run"))]
pub mod freebsd;

#[cfg(all(in_os_platform = "macos", feature = "local-run"))]
pub mod macos;

#[cfg(all(in_os_platform = "redhat", feature = "local-run"))]
pub mod redhat;

#[cfg(all(in_os_platform = "ubuntu", feature = "local-run"))]
pub mod ubuntu;

pub struct Target;
