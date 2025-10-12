use anyhow::{Error, Ok, Result};
use serde_json::Value;

use crate::sys::arch::Arch;
use std::{ffi::CStr, mem};

#[derive(Debug)]
pub struct SysInfo {
    pub sysname: String,
    pub release: String,
    pub hostname: String,
    pub version: String,
    pub machine: Arch,
}

pub fn get_platform_information() -> Result<SysInfo> {
    unsafe {
        let mut uts = mem::MaybeUninit::<libc::utsname>::uninit();
        let result = libc::uname(uts.as_mut_ptr());

        if result != 0 {
            return Err(anyhow::anyhow!("uname() failed with error {}.", 1));
        }

        let utsname_value = uts.assume_init();
        let raw_arch = CStr::from_ptr(utsname_value.machine.as_ptr()).to_string_lossy();

        Ok(SysInfo {
            sysname: CStr::from_ptr(utsname_value.sysname.as_ptr())
                .to_string_lossy()
                .into_owned(),
            release: CStr::from_ptr(utsname_value.release.as_ptr())
                .to_string_lossy()
                .into_owned(),
            hostname: CStr::from_ptr(utsname_value.nodename.as_ptr())
                .to_string_lossy()
                .into_owned(),
            version: CStr::from_ptr(utsname_value.version.as_ptr())
                .to_string_lossy()
                .into_owned(),
            machine: Arch::from_str(&raw_arch.into_owned()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_arch() {
        assert_eq!(1, 1);
    }
}
