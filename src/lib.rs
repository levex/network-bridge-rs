//! This crate is a simple wrapper around the ioctls exposed by Linux.
//!
//! The goal of this crate is to make it easy to create networking bridges in Rust.
#[macro_use]
extern crate nix;
use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag};

extern crate libc;

use std::ffi::CString;

mod private {
    extern crate libc;
    /// The maximum length of an interface name.
    pub const IFNAMSIZ: usize = 16;
    pub const SIOCBRADDBR: u16 = 0x89a0;
    pub const SIOCBRDELBR: u16 = 0x89a1;
    pub const SIOCGIFINDEX: u16 = 0x8933;
    pub const SIOCBRADDIF: u16 = 0x89a2;
    pub const SIOCBRDELIF: u16 = 0x89a3;

    #[repr(C)]
    pub struct ifreq {
       pub ifrn_name: [libc::c_char; IFNAMSIZ],
       pub ifru_ivalue: u32,
    }

    ioctl_write_ptr_bad!(ioctl_addbr, SIOCBRADDBR, libc::c_char);
    ioctl_write_ptr_bad!(ioctl_delbr, SIOCBRDELBR, libc::c_char);
    ioctl_write_ptr_bad!(ioctl_ifindex, SIOCGIFINDEX, ifreq);
    ioctl_write_ptr_bad!(ioctl_addif, SIOCBRADDIF, ifreq);
    ioctl_write_ptr_bad!(ioctl_delif, SIOCBRDELIF, ifreq);
}
use crate::private::{ifreq, ioctl_addif, ioctl_delif, ioctl_ifindex, ioctl_delbr, ioctl_addbr};
pub use crate::private::IFNAMSIZ;

/// Builder pattern for constructing networking bridges.
///
/// # Example
/// 
/// Create a bridge named `hello_world_br` and attach two interfaces: `eth0` and `eth1`.
///
/// ```rust,no_run
///# use ::network_bridge::BridgeBuilder;
///   let result = BridgeBuilder::new("hello_world_br")
///                 .interface("eth0")
///                 .interface("eth1")
///                 .build();
/// ```
pub struct BridgeBuilder {
    name: String,
    interfaces: Vec<i32>,
}

impl BridgeBuilder {
    /// Start building a new bridge, setting its interface name.
    pub fn new(name: &str) -> BridgeBuilder {
        BridgeBuilder {
            name: name.to_string(),
            interfaces: Vec::new(),
        }
    }

    /// Override the name of the bridge.
    pub fn name(self, name: &str) -> BridgeBuilder {
        BridgeBuilder {
            name: name.to_string(),
            interfaces: self.interfaces,
        }
    }

    /// Attach an interface to the bridge.
    /// 
    /// Note that this will fail _silently_ if the interface name supplied cannot be converted into
    /// the appropriate interface index.
    pub fn interface(self, name: &str) -> BridgeBuilder {
        let idx = interface_id(name);
        if idx.is_ok() {
            BridgeBuilder {
                name: self.name,
                interfaces: {
                    let mut ifs = self.interfaces;
                    ifs.push(idx.unwrap());
                    ifs
                }
            }
        } else {
            self
        }
    }

    /// Remove an interface from the bridge.
    ///
    /// Note that this will fail _silently_ if the interface name supplied cannot be converted into
    /// the appropriate interface index.
    pub fn remove_interface(self, name: &str) -> BridgeBuilder {
        let idx = interface_id(name);
        if idx.is_ok() {
            BridgeBuilder {
                name: self.name,
                interfaces:
                    self.interfaces.into_iter()
                        .filter(|x| *x != idx.unwrap())
                        .collect()
                ,
            }
        } else {
            self
        }
    }

    /// Finalize the builder, creating the bridge and attaching any interfaces.
    pub fn build(self) -> Result<(), nix::Error> {
        create_bridge(&self.name)?;
        for i in self.interfaces {
            add_interface_to_bridge(i, &self.name)?;
        }

        Ok(())
    }
}

/// Create a network bridge using the interface name supplied.
pub fn create_bridge(name: &str) -> Result<i32, nix::Error> {
    /* Open a socket */
    let res = socket(AddressFamily::Unix,
                     SockType::Stream,
                     SockFlag::empty(),
                     None)?;

    /* use the SIOCBRADDRBR ioctl to add the bridge */
    let cstr = CString::new(name).unwrap();
    unsafe {
        ioctl_addbr(res, cstr.as_ptr())
    }
}

/// Delete an existing network bridge of the interface name supplied.
pub fn delete_bridge(name: &str) -> Result<i32, nix::Error> {
    /* Open a socket */
    let res = socket(AddressFamily::Unix,
                     SockType::Stream,
                     SockFlag::empty(),
                     None)?;

    /* use the SIOCBRDELBR ioctl to delete the bridge */
    let cstr = CString::new(name).unwrap();
    unsafe {
        ioctl_delbr(res, cstr.as_ptr())
    }
}

/// Converts an interface name into the identifier used by the kernel.
///
/// This can also be retrieved via sysfs, if mounted to /sys:
///
/// ```shell
/// $ cat /sys/class/net/eth0/ifindex
/// 1
/// ```
pub fn interface_id(interface: &str) -> Result<i32, nix::Error> {
    /* do some validation */
    if interface.len() == 0 || interface.len() >= IFNAMSIZ {
        return Err(nix::Error::from(nix::errno::Errno::EINVAL));
    }
    let length = interface.len();

    /* Open a socket */
    let sock = socket(AddressFamily::Unix,
                     SockType::Stream,
                     SockFlag::empty(),
                     None)?;

    let cstr = CString::new(interface).unwrap();

    /* create the ifreq structure */
    let ifr = ifreq {
        ifrn_name: [0; IFNAMSIZ],
        ifru_ivalue: 0,
    };

    let result = unsafe {
         /*
          * This is safe because length is guaranteed to be less than IFNAMSIZ,
          * and the two variables can never overlap 
          */
        std::ptr::copy_nonoverlapping(cstr.as_ptr(),
            ifr.ifrn_name.as_ptr() as *mut i8, length);
        /*
         * SIOCGIFINDEX doesn't care about the rest of the fields, so this
         * should be safe
         */
        ioctl_ifindex(sock, &ifr)
    };

    if result.is_err() {
        result
    } else {
        Ok(ifr.ifru_ivalue as i32)
    }
}

fn bridge_del_add_if(interface_id: i32, bridge: &str, add: bool)
    -> Result<i32, nix::Error> {
    /* validate bridge name */
    if bridge.len() == 0 || bridge.len() >= IFNAMSIZ {
        return Err(nix::Error::from(nix::errno::Errno::EINVAL));
    }
    let length = bridge.len();

    /* Open a socket */
    let sock = socket(AddressFamily::Unix,
                     SockType::Stream,
                     SockFlag::empty(),
                     None)?;

    let ifr = ifreq {
        ifrn_name: [0; IFNAMSIZ],
        ifru_ivalue: interface_id as u32,
    };

    let br_cstr = CString::new(bridge).unwrap();

    unsafe {
        /* copy the bridge name to the ifreq */
        std::ptr::copy_nonoverlapping(br_cstr.as_ptr(),
            ifr.ifrn_name.as_ptr() as *mut i8, length);

        if add {
            ioctl_addif(sock, &ifr)
        } else {
            ioctl_delif(sock, &ifr)
        }
    }
}

/// Attach an interface to a bridge.
///
/// The bridge must already exist.
pub fn add_interface_to_bridge(interface_id: i32, bridge: &str)
    -> Result<i32, nix::Error> {
    bridge_del_add_if(interface_id, bridge, true)
}

/// Remove an interface from a bridge.
///
/// The bridge must already exist and the interface must already be attached to the bridge.
pub fn delete_interface_from_bridge(interface_id: i32, bridge: &str)
    -> Result<i32, nix::Error> {
    bridge_del_add_if(interface_id, bridge, false)
}

#[cfg(test)]
mod tests {
    use crate::{create_bridge, delete_bridge, interface_id, add_interface_to_bridge};

    #[test]
    fn add_and_delete_bridge() {
        assert!(create_bridge("hello_br0").is_ok());
        assert!(delete_bridge("hello_br0").is_ok());
    }

    #[test]
    fn get_interface_id() {
        assert!(interface_id("eth0").is_ok());
    }

    #[test]
    fn test_adding_to_bridge() {
        assert!(create_bridge("hello_br1").is_ok());
        assert!(add_interface_to_bridge(interface_id("eth0").unwrap(),
                    "hello_br1").is_ok());
        assert!(delete_bridge("hello_br1").is_ok());
    }
}
