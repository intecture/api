// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Host

use ffi_helpers::Ffi__Array;
use host::Host;
use host::ffi::Ffi__Host;
use libc::{c_char, c_float, uint8_t, uint32_t, uint64_t};
use std::{convert, ptr};
use std::ffi::CString;
use std::panic::catch_unwind;
use super::*;

#[repr(C)]
pub struct Ffi__Telemetry {
    pub cpu: Ffi__Cpu,
    pub fs: Ffi__Array<Ffi__FsMount>,
    pub hostname: *const c_char,
    pub memory: uint64_t,
    pub net: Ffi__Array<Ffi__Netif>,
    pub os: Ffi__Os,
}

impl convert::From<Telemetry> for Ffi__Telemetry {
    fn from(telemetry: Telemetry) -> Ffi__Telemetry {
        let mut fs = vec![];
        for mount in telemetry.fs {
            fs.push(Ffi__FsMount::from(mount));
        }

        let mut net = vec![];
        for netif in telemetry.net {
            let n = Ffi__Netif::from(netif);
            net.push(n);
        }

        Ffi__Telemetry {
            cpu: Ffi__Cpu::from(telemetry.cpu),
            fs: Ffi__Array::from(fs),
            hostname: CString::new(telemetry.hostname).unwrap().into_raw(),
            memory: telemetry.memory as uint64_t,
            net: Ffi__Array::from(net),
            os: Ffi__Os::from(telemetry.os),
        }
    }
}

impl convert::Into<Telemetry> for Ffi__Telemetry {
    fn into(self) -> Telemetry {
        let ffi_fs: Vec<_> = self.fs.into();
        let mut fs: Vec<FsMount> = vec![];
        for mount in ffi_fs {
            fs.push(mount.into());
        }

        let ffi_net: Vec<_> = self.net.into();
        let mut net: Vec<Netif> = vec![];
        for iface in ffi_net {
            net.push(iface.into());
        }

        Telemetry {
            cpu: self.cpu.into(),
            fs: fs,
            hostname: trypanic!(ptrtostr!(self.hostname, "hostname string")).into(),
            memory: self.memory as u64,
            net: net,
            os: self.os.into(),
        }
    }
}

#[repr(C)]
pub struct Ffi__Cpu {
    pub vendor: *const c_char,
    pub brand_string: *const c_char,
    pub cores: uint32_t,
}

impl convert::From<Cpu> for Ffi__Cpu {
    fn from(cpu: Cpu) -> Ffi__Cpu {
        Ffi__Cpu {
            vendor: CString::new(cpu.vendor).unwrap().into_raw(),
            brand_string: CString::new(cpu.brand_string).unwrap().into_raw(),
            cores: cpu.cores as uint32_t,
        }
    }
}

impl convert::Into<Cpu> for Ffi__Cpu {
    fn into(self) -> Cpu {
        Cpu {
            vendor: trypanic!(ptrtostr!(self.vendor, "vendor string")).into(),
            brand_string: trypanic!(ptrtostr!(self.brand_string, "brand string")).into(),
            cores: self.cores,
        }
    }
}

#[repr(C)]
pub struct Ffi__FsMount {
    pub filesystem: *const c_char,
    pub mountpoint: *const c_char,
    pub size: uint64_t,
    pub used: uint64_t,
    pub available: uint64_t,
    pub capacity: c_float,
}

impl convert::From<FsMount> for Ffi__FsMount {
    fn from(mount: FsMount) -> Ffi__FsMount {
        Ffi__FsMount {
            filesystem: CString::new(mount.filesystem).unwrap().into_raw(),
            mountpoint: CString::new(mount.mountpoint).unwrap().into_raw(),
            size: mount.size,
            used: mount.used,
            available: mount.available,
            capacity: mount.capacity,
        }
    }
}

impl convert::Into<FsMount> for Ffi__FsMount {
    fn into(self) -> FsMount {
        FsMount {
            filesystem: trypanic!(ptrtostr!(self.filesystem, "filesystem string")).into(),
            mountpoint: trypanic!(ptrtostr!(self.mountpoint, "mountpoint string")).into(),
            size: self.size,
            used: self.used,
            available: self.available,
            capacity: self.capacity,
        }
    }
}

#[repr(C)]
pub struct Ffi__Netif {
    pub interface: *const c_char,
    pub mac: *const c_char,
    pub inet: *const Ffi__NetifIPv4,
    pub inet6: *const Ffi__NetifIPv6,
    pub status: *const c_char,
}

impl convert::From<Netif> for Ffi__Netif {
    fn from(netif: Netif) -> Ffi__Netif {
        Ffi__Netif {
            interface: CString::new(netif.interface).unwrap().into_raw(),
            mac: match netif.mac {
                Some(mac) => CString::new(mac).unwrap().into_raw(),
                None => ptr::null(),
            },
            inet: match netif.inet {
                Some(inet) => Box::into_raw(Box::new(Ffi__NetifIPv4::from(inet))),
                None => ptr::null(),
            },
            inet6: match netif.inet6 {
                Some(inet6) => Box::into_raw(Box::new(Ffi__NetifIPv6::from(inet6))),
                None => ptr::null(),
            },
            status: match netif.status {
                Some(status) => match status {
                    NetifStatus::Active => CString::new("Active").unwrap().into_raw(),
                    NetifStatus::Inactive => CString::new("Inactive").unwrap().into_raw(),
                },
                None => ptr::null(),
            },
        }
    }
}

impl convert::Into<Netif> for Ffi__Netif {
    fn into(self) -> Netif {
        Netif {
            interface: trypanic!(ptrtostr!(self.interface, "interface string")).into(),
            mac: if self.mac.is_null() {
                None
            } else {
                Some(trypanic!(ptrtostr!(self.mac, "mac string")).into())
            },
            inet: if self.inet.is_null() {
                None
            } else {
                Some(trypanic!(readptr!(self.inet, "NetifIPv4 struct")))
            },
            inet6: if self.inet6.is_null() {
                None
            } else {
                Some(trypanic!(readptr!(self.inet6, "NetifIPv6 struct")))
            },
            status: if self.status.is_null() {
                None
            } else {
                Some(match trypanic!(ptrtostr!(self.status, "NetifStatus struct")) {
                    "Active" => NetifStatus::Active,
                    "Inactive" => NetifStatus::Inactive,
                    _ => unreachable!(),
                })
            },
        }
    }
}

#[repr(C)]
pub struct Ffi__NetifIPv4 {
    pub address: *const c_char,
    pub netmask: *const c_char,
}

impl convert::From<NetifIPv4> for Ffi__NetifIPv4 {
    fn from(netif: NetifIPv4) -> Ffi__NetifIPv4 {
        Ffi__NetifIPv4 {
            address: CString::new(netif.address).unwrap().into_raw(),
            netmask: CString::new(netif.netmask).unwrap().into_raw(),
        }
    }
}

impl convert::Into<NetifIPv4> for Ffi__NetifIPv4 {
    fn into(self) -> NetifIPv4 {
        NetifIPv4 {
            address: trypanic!(ptrtostr!(self.address, "address string")).into(),
            netmask: trypanic!(ptrtostr!(self.netmask, "netmask string")).into(),
        }
    }
}

#[repr(C)]
pub struct Ffi__NetifIPv6 {
    pub address: *const c_char,
    pub prefixlen: uint8_t,
    pub scopeid: *const c_char,
}

impl convert::From<NetifIPv6> for Ffi__NetifIPv6 {
    fn from(netif: NetifIPv6) -> Ffi__NetifIPv6 {
        Ffi__NetifIPv6 {
            address: CString::new(netif.address).unwrap().into_raw(),
            prefixlen: netif.prefixlen,
            scopeid: if netif.scopeid.is_some() {
                CString::new(netif.scopeid.unwrap()).unwrap().into_raw()
            } else {
                ptr::null()
            },
        }
    }
}

impl convert::Into<NetifIPv6> for Ffi__NetifIPv6 {
    fn into(self) -> NetifIPv6 {
        NetifIPv6 {
            address: trypanic!(ptrtostr!(self.address, "address string")).into(),
            prefixlen: self.prefixlen,
            scopeid: if self.scopeid.is_null() {
                None
            } else {
                Some(trypanic!(ptrtostr!(self.scopeid, "scopeid string")).into())
            },
        }
    }
}

#[repr(C)]
pub struct Ffi__Os {
    pub arch: *mut c_char,
    pub family: *mut c_char,
    pub platform: *mut c_char,
    pub version: *mut c_char,
}

impl convert::From<Os> for Ffi__Os {
    fn from(os: Os) -> Ffi__Os {
        Ffi__Os {
            arch: CString::new(os.arch).unwrap().into_raw(),
            family: CString::new(os.family).unwrap().into_raw(),
            platform: CString::new(os.platform).unwrap().into_raw(),
            version: CString::new(os.version).unwrap().into_raw(),
        }
    }
}

impl convert::Into<Os> for Ffi__Os {
    fn into(self) -> Os {
        Os {
            arch: trypanic!(ptrtostr!(self.arch, "arch string")).into(),
            family: trypanic!(ptrtostr!(self.family, "family string")).into(),
            platform: trypanic!(ptrtostr!(self.platform, "platform string")).into(),
            version: trypanic!(ptrtostr!(self.version, "version string")).into(),
        }
    }
}

#[no_mangle]
pub extern "C" fn telemetry_init(host_ptr: *mut Ffi__Host) -> *const Ffi__Telemetry {
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));
    let telemetry = trynull!(Telemetry::init(&mut host));
    let ffi_t = trynull!(catch_unwind(|| telemetry.into()));
    Box::into_raw(Box::new(ffi_t))
}

#[no_mangle]
pub extern "C" fn telemetry_free(telemetry_ptr: *mut Ffi__Telemetry) -> uint8_t {
    let _: Telemetry = tryrc!(readptr!(telemetry_ptr, "Telemetry struct"));
    0
}

#[cfg(test)]
mod tests {
    use ffi_helpers::Ffi__Array;
    use libc::{c_float, uint64_t};
    use std::ffi::CString;
    use std::mem;
    use super::*;
    use super::super::*;

    #[test]
    fn test_convert_telemetry() {
        let _: Ffi__Telemetry = create_telemetry().into();
        let _: Telemetry = create_ffi_telemetry().into();
    }

    #[test]
    fn test_telemetry_free() {
        telemetry_free(&mut create_ffi_telemetry() as *mut Ffi__Telemetry);
    }

    fn create_telemetry() -> Telemetry {
        Telemetry {
            cpu: Cpu {
                vendor: "moo".to_string(),
                brand_string: "Moo Cow Super Fun Happy CPU".to_string(),
                cores: 100,
            },
            fs: vec![FsMount {
                filesystem: "/dev/disk0".to_string(),
                mountpoint: "/".to_string(),
                size: 10000,
                used: 5000,
                available: 5000,
                capacity: 0.5,
                // inodes_used: 20,
                // inodes_available: 0,
                // inodes_capacity: 1.0,
            }],
            hostname: "localhost".to_string(),
            memory: 2048,
            net: vec![Netif {
                interface: "em0".to_string(),
                mac: Some("01:23:45:67:89:ab".to_string()),
                inet: Some(NetifIPv4 {
                    address: "127.0.0.1".to_string(),
                    netmask: "255.255.255.255".to_string(),
                }),
                inet6: Some(NetifIPv6 {
                    address: "::1".to_string(),
                    prefixlen: 8,
                    scopeid: Some("0x4".to_string()),
                }),
                status: Some(NetifStatus::Active),
            }],
            os: Os {
                arch: "doctor string".to_string(),
                family: "moo".to_string(),
                platform: "cow".to_string(),
                version: "1.0".to_string(),
            },
        }
    }

    fn create_ffi_telemetry() -> Ffi__Telemetry {
        let mut fs = vec![Ffi__FsMount {
            filesystem: CString::new("/dev/disk0").unwrap().into_raw(),
            mountpoint: CString::new("/").unwrap().into_raw(),
            size: 10000 as uint64_t,
            used: 5000 as uint64_t,
            available: 5000 as uint64_t,
            capacity: 0.5 as c_float,
//            inodes_used: 20 as uint64_t,
//            inodes_available: 0 as uint64_t,
//            inodes_capacity: 1.0 as c_float,
        }];

        let mut net = vec![Ffi__Netif {
            interface: CString::new("em0").unwrap().into_raw(),
            mac: CString::new("01:23:45:67:89:ab").unwrap().into_raw(),
            inet: Box::into_raw(Box::new(Ffi__NetifIPv4 {
                address: CString::new("01:23:45:67:89:ab").unwrap().into_raw(),
                netmask: CString::new("255.255.255.255").unwrap().into_raw(),
            })),
            inet6: Box::into_raw(Box::new(Ffi__NetifIPv6 {
                address: CString::new("::1").unwrap().into_raw(),
                prefixlen: 8,
                scopeid: CString::new("0x4").unwrap().into_raw(),
            })),
            status: CString::new("Active").unwrap().into_raw(),
        }];

        let ffi_telemetry = Ffi__Telemetry {
            cpu: Ffi__Cpu {
                vendor: CString::new("moo").unwrap().into_raw(),
                brand_string: CString::new("Moo Cow Super Fun Happy CPU").unwrap().into_raw(),
                cores: 100,
            },
            fs: Ffi__Array {
                ptr: fs.as_mut_ptr(),
                length: fs.len(),
                capacity: fs.capacity(),
            },
            hostname: CString::new("localhost").unwrap().into_raw(),
            memory: 2048,
            net: Ffi__Array {
                ptr: net.as_mut_ptr(),
                length: net.len(),
                capacity: net.capacity(),
            },
            os: Ffi__Os {
                arch: CString::new("doctor string").unwrap().into_raw(),
                family: CString::new("moo").unwrap().into_raw(),
                platform: CString::new("cow").unwrap().into_raw(),
                version: CString::new("1.0").unwrap().into_raw(),
            },
        };

        // Note: This causes a memory leak but unless we forget them,
        // Rust will deallocate the memory and Telemetry::from() will
        // segfault.
        mem::forget(fs);
        mem::forget(net);

        ffi_telemetry
    }
}
