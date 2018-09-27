use core::slice;
//use core::ffi::CStr;
//use core::os::raw;

pub const MULTIBOOT_BOOT_MAGIC: u32 = 0x2BADB002;

#[derive(Debug)]
#[repr(C)]
pub struct MultibootInfo {
    flags: u32,
    mem_info: MultibootMemInfo,
    _boot_device: u32,
    cmdline: *const u8,
}

impl MultibootInfo {
    pub fn mem_info(&self) -> Option<&MultibootMemInfo> {
        if self.flags & 1 != 0 {
            Some(&self.mem_info)
        } else {
            None
        }
    }

    pub fn cmdline(&self) -> Option<&str> {
        if self.flags & 0x4 != 0 {
            let str_slice = unsafe {
                slice::from_raw_parts(self.cmdline, strlen(self.cmdline))
            };
            Some(unsafe { ::core::str::from_utf8_unchecked(str_slice) })  // TODO Check!
        } else {
            None
        }
    }

}

#[derive(Debug)]
#[repr(C)]
pub struct MultibootMemInfo {
    mem_lower: u32,
    mem_upper: u32,
}

unsafe fn strlen(c: *const u8) -> usize {
    let mut len: usize = 0;
    while *c.offset(len as isize) != 0 {
        len += 1;
    }

    len
}
