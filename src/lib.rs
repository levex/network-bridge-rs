#[macro_use]
extern crate nix;
use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag};

extern crate libc;
use libc::c_char;

use std::ffi::CString;

const SIOCBRADDBR: u16 = 0x89a0;
ioctl_write_ptr_bad!(ioctl_addbr, SIOCBRADDBR, c_char);

pub fn add_bridge(name: String) -> Result<i32, nix::Error> {
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

#[cfg(test)]
mod tests {
    use add_bridge;

    #[test]
    fn check_add_bridge() {
        add_bridge("hello_br0".to_string());
    }
}
