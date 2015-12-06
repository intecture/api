// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Host

use host::Host;
use host::ffi::Ffi__Host;
use libc::{c_char, c_float, size_t, uint32_t, uint64_t};
use std::{convert, mem, ptr};
use std::ffi::{CStr, CString};
use super::*;

#[repr(C)]
pub struct Ffi__Telemetry {
    pub cpu: Ffi__Cpu,
    pub fs: Ffi__Array<Ffi__FsMount>,
    pub hostname: *mut c_char,
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

impl convert::From<Ffi__Telemetry> for Telemetry {
    fn from(ffi_telemetry: Ffi__Telemetry) -> Telemetry {
        let fs_vec = unsafe { Vec::from_raw_parts(ffi_telemetry.fs.ptr, ffi_telemetry.fs.length, ffi_telemetry.fs.capacity) };
        let mut fs = vec![];
        for mount in fs_vec {
            fs.push(FsMount::from(mount));
        }

        let net_vec = unsafe { Vec::from_raw_parts(ffi_telemetry.net.ptr, ffi_telemetry.net.length, ffi_telemetry.net.capacity) };
        let mut net = vec![];
        for iface in net_vec {
            net.push(Netif::from(iface));
        }

        Telemetry {
            cpu: Cpu::from(ffi_telemetry.cpu),
            fs: fs,
            hostname: unsafe { CString::from_raw(ffi_telemetry.hostname) }.to_str().unwrap().to_string(),
            memory: ffi_telemetry.memory as u64,
            net: net,
            os: Os::from(ffi_telemetry.os),
        }
    }
}

#[repr(C)]
pub struct Ffi__Cpu {
    pub vendor: *mut c_char,
    pub brand_string: *mut c_char,
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

impl convert::From<Ffi__Cpu> for Cpu {
    fn from(ffi_cpu: Ffi__Cpu) -> Cpu {
        Cpu {
            vendor: unsafe { CString::from_raw(ffi_cpu.vendor) }.to_str().unwrap().to_string(),
            brand_string: unsafe { CString::from_raw(ffi_cpu.brand_string) }.to_str().unwrap().to_string(),
            cores: ffi_cpu.cores as u32,
        }
    }
}

#[repr(C)]
pub struct Ffi__FsMount {
    pub filesystem: *mut c_char,
    pub mountpoint: *mut c_char,
    pub size: uint64_t,
    pub used: uint64_t,
    pub available: uint64_t,
    pub capacity: c_float,
    pub inodes_used: uint64_t,
    pub inodes_available: uint64_t,
    pub inodes_capacity: c_float,
}

impl convert::From<FsMount> for Ffi__FsMount {
    fn from(mount: FsMount) -> Ffi__FsMount {
        Ffi__FsMount {
            filesystem: CString::new(mount.filesystem).unwrap().into_raw(),
            mountpoint: CString::new(mount.mountpoint).unwrap().into_raw(),
            size: mount.size as uint64_t,
            used: mount.used as uint64_t,
            available: mount.available as uint64_t,
            capacity: mount.capacity as c_float,
            inodes_used: mount.inodes_used as uint64_t,
            inodes_available: mount.inodes_available as uint64_t,
            inodes_capacity: mount.inodes_capacity as c_float,
        }
    }
}

impl convert::From<Ffi__FsMount> for FsMount {
    fn from(ffi_mount: Ffi__FsMount) -> FsMount {
        FsMount {
            filesystem: unsafe { CString::from_raw(ffi_mount.filesystem) }.to_str().unwrap().to_string(),
            mountpoint: unsafe { CString::from_raw(ffi_mount.mountpoint) }.to_str().unwrap().to_string(),
            size: ffi_mount.size as u64,
            used: ffi_mount.used as u64,
            available: ffi_mount.available as u64,
            capacity: ffi_mount.capacity as f32,
            inodes_used: ffi_mount.inodes_used as u64,
            inodes_available: ffi_mount.inodes_available as u64,
            inodes_capacity: ffi_mount.inodes_capacity as f32,
        }
    }
}

#[repr(C)]
pub struct Ffi__Netif {
    pub interface: *mut c_char,
    pub mac: *mut c_char,
    pub inet: Ffi__NetifIPv4,
    pub inet6: Ffi__NetifIPv6,
    pub status: *mut c_char,
}

impl convert::From<Netif> for Ffi__Netif {
    fn from(netif: Netif) -> Ffi__Netif {
        Ffi__Netif {
            interface: CString::new(netif.interface).unwrap().into_raw(),
            mac: if netif.mac.is_some() {
                    CString::new(netif.mac.unwrap()).unwrap().into_raw()
                } else {
                    CString::new("").unwrap().into_raw()
                },
            inet: if netif.inet.is_some() {
                    Ffi__NetifIPv4::from(netif.inet.unwrap())
                } else {
                    Ffi__NetifIPv4::from(NetifIPv4::new(String::new(), String::new()))
                },
            inet6: if netif.inet6.is_some() {
                    Ffi__NetifIPv6::from(netif.inet6.unwrap())
                } else {
                    Ffi__NetifIPv6::from(NetifIPv6::new(String::new(), 0, None))
                },
            status: if netif.status.is_some() {
                    match netif.status.unwrap() {
                        NetifStatus::Active => CString::new("Active").unwrap().into_raw(),
                        NetifStatus::Inactive => CString::new("Inactive").unwrap().into_raw(),
                    }
                } else {
                    CString::new("").unwrap().into_raw()
                },
        }
    }
}

impl convert::From<Ffi__Netif> for Netif {
    fn from(ffi_netif: Ffi__Netif) -> Netif {
        Netif {
            interface: unsafe { CStr::from_ptr(ffi_netif.interface) }.to_str().unwrap().to_string(),
            mac: {
                let mac = unsafe { CStr::from_ptr(ffi_netif.mac) }.to_str().unwrap();
                if mac == "" {
                    None
                } else {
                    Some(mac.to_string())
                }
            },
            inet: {
                let ipv4 = NetifIPv4::from(ffi_netif.inet);
                if ipv4.address == "" {
                    None
                } else {
                    Some(ipv4)
                }
            },
            inet6: {
                let ipv6 = NetifIPv6::from(ffi_netif.inet6);
                if ipv6.address == "" {
                    None
                } else {
                    Some(ipv6)
                }
            },
            status: {
                let status = unsafe { CStr::from_ptr(ffi_netif.status) }.to_str().unwrap();
                match status {
                    "Active" => Some(NetifStatus::Active),
                    "Inactive" => Some(NetifStatus::Inactive),
                    _ => None,
                }
            }
        }
    }
}

#[repr(C)]
pub struct Ffi__NetifIPv4 {
    pub address: *mut c_char,
    pub netmask: *mut c_char,
}

impl convert::From<NetifIPv4> for Ffi__NetifIPv4 {
    fn from(netif: NetifIPv4) -> Ffi__NetifIPv4 {
        Ffi__NetifIPv4 {
            address: CString::new(netif.address).unwrap().into_raw(),
            netmask: CString::new(netif.netmask).unwrap().into_raw(),
        }
    }
}

impl convert::From<Ffi__NetifIPv4> for NetifIPv4 {
    fn from(ffi_netif: Ffi__NetifIPv4) -> NetifIPv4 {
        NetifIPv4 {
            address: unsafe { CStr::from_ptr(ffi_netif.address) }.to_str().unwrap().to_string(),
            netmask: unsafe { CStr::from_ptr(ffi_netif.netmask) }.to_str().unwrap().to_string(),
        }
    }
}

#[repr(C)]
pub struct Ffi__NetifIPv6 {
    pub address: *mut c_char,
    pub prefixlen: uint32_t,
    pub scopeid: *mut c_char,
}

impl convert::From<NetifIPv6> for Ffi__NetifIPv6 {
    fn from(netif: NetifIPv6) -> Ffi__NetifIPv6 {
        Ffi__NetifIPv6 {
            address: CString::new(netif.address).unwrap().into_raw(),
            prefixlen: netif.prefixlen as uint32_t,
            scopeid: if netif.scopeid.is_some() {
                CString::new(netif.scopeid.unwrap()).unwrap().into_raw()
            } else {
                CString::new("").unwrap().into_raw()
            },
        }
    }
}

impl convert::From<Ffi__NetifIPv6> for NetifIPv6 {
    fn from(netif: Ffi__NetifIPv6) -> NetifIPv6 {
        NetifIPv6 {
            address: unsafe { CStr::from_ptr(netif.address) }.to_str().unwrap().to_string(),
            prefixlen: netif.prefixlen as u8,
            scopeid: {
                let scopeid = unsafe { CStr::from_ptr(netif.scopeid) }.to_str().unwrap().to_string();
                if scopeid == "" {
                    None
                } else {
                    Some(scopeid)
                }
            }
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

impl convert::From<Ffi__Os> for Os {
    fn from(os: Ffi__Os) -> Os {
        Os {
            arch: unsafe { CStr::from_ptr(os.arch) }.to_str().unwrap().to_string(),
            family: unsafe { CStr::from_ptr(os.family) }.to_str().unwrap().to_string(),
            platform: unsafe { CStr::from_ptr(os.platform) }.to_str().unwrap().to_string(),
            version: unsafe { CStr::from_ptr(os.version) }.to_str().unwrap().to_string(),
        }
    }
}

#[repr(C)]
pub struct Ffi__Array<T> {
    pub ptr: *mut T,
    pub length: size_t,
    pub capacity: size_t,
}

impl <T>convert::From<Vec<T>> for Ffi__Array<T> {
    fn from(item: Vec<T>) -> Ffi__Array<T> {
        let mut item = item;

        item.shrink_to_fit();

        let ffi_item = Ffi__Array {
            ptr: item.as_mut_ptr(),
            length: item.len() as size_t,
            capacity: item.capacity() as size_t,
        };

        mem::forget(item);

        ffi_item
    }
}

#[no_mangle]
pub extern "C" fn telemetry_init(ffi_host_ptr: *mut Ffi__Host) -> Ffi__Telemetry {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let telemetry = Ffi__Telemetry::from(Telemetry::init(&mut host).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    telemetry
}

#[no_mangle]
pub extern "C" fn telemetry_free(ffi_telemetry_ptr: *mut Ffi__Telemetry) {
    // Once converted from raw pointers to Rust pointers, we can just
    // let the value fall out of scope to free.
    Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });
}

#[cfg(test)]
mod tests {
    use libc::{c_float, size_t, uint32_t, uint64_t};
    use std::ffi::CString;
    use std::mem;
    use super::*;
    use super::super::*;

    #[test]
    fn test_convert_telemetry() {
        Ffi__Telemetry::from(create_telemetry());
    }

    #[test]
    fn test_convert_ffi_telemetry() {
        Telemetry::from(create_ffi_telemetry());
    }

    #[test]
    fn test_telemetry_free() {
        telemetry_free(&mut create_ffi_telemetry() as *mut Ffi__Telemetry);
    }

    fn create_telemetry() -> Telemetry {
        Telemetry::new(
            Cpu::new(
                "moo".to_string(),
                "Moo Cow Super Fun Happy CPU".to_string(),
                100,
            ),
            vec![FsMount::new(
                "/dev/disk0".to_string(),
                "/".to_string(),
                10000,
                5000,
                5000,
                0.5,
                20,
                0,
                1.0,
            )],
            "localhost".to_string(),
            2048,
            vec![Netif::new(
                "em0".to_string(),
                Some("01:23:45:67:89:ab".to_string()),
                Some(NetifIPv4::new(
                    "127.0.0.1".to_string(),
                    "255.255.255.255".to_string(),
                )),
                Some(NetifIPv6::new(
                    "::1".to_string(),
                    8,
                    Some("0x4".to_string()),
                )),
                Some(NetifStatus::Active),
            )],
            Os::new(
                "doctor string".to_string(),
                "moo".to_string(),
                "cow".to_string(),
                "1.0".to_string(),
            ),
        )
    }

    fn create_ffi_telemetry() -> Ffi__Telemetry {
        let mut fs = vec![Ffi__FsMount {
            filesystem: CString::new("/dev/disk0").unwrap().into_raw(),
            mountpoint: CString::new("/").unwrap().into_raw(),
            size: 10000 as uint64_t,
            used: 5000 as uint64_t,
            available: 5000 as uint64_t,
            capacity: 0.5 as c_float,
            inodes_used: 20 as uint64_t,
            inodes_available: 0 as uint64_t,
            inodes_capacity: 1.0 as c_float,
        }];

        let mut net = vec![Ffi__Netif {
            interface: CString::new("em0").unwrap().into_raw(),
            mac: CString::new("01:23:45:67:89:ab").unwrap().into_raw(),
            inet: Ffi__NetifIPv4 {
                address: CString::new("01:23:45:67:89:ab").unwrap().into_raw(),
                netmask: CString::new("255.255.255.255").unwrap().into_raw(),
            },
            inet6: Ffi__NetifIPv6 {
                address: CString::new("::1").unwrap().into_raw(),
                prefixlen: 8 as uint32_t,
                scopeid: CString::new("0x4").unwrap().into_raw(),
            },
            status: CString::new("Active").unwrap().into_raw(),
        }];

        let ffi_telemetry = Ffi__Telemetry {
            cpu: Ffi__Cpu {
                vendor: CString::new("moo").unwrap().into_raw(),
                brand_string: CString::new("Moo Cow Super Fun Happy CPU").unwrap().into_raw(),
                cores: 100 as uint32_t,
            },
            fs: Ffi__Array {
                ptr: fs.as_mut_ptr(),
                length: fs.len() as size_t,
                capacity: fs.capacity() as size_t,
            },
            hostname: CString::new("localhost").unwrap().into_raw(),
            memory: 2048 as uint64_t,
            net: Ffi__Array {
                ptr: net.as_mut_ptr(),
                length: net.len() as size_t,
                capacity: net.capacity() as size_t,
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