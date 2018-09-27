use core::marker::PhantomData;
use core::slice;
//use core::ffi::CStr;
use bitfield::*;

pub const MULTIBOOT_BOOT_MAGIC: u32 = 0x2BADB002;

bitfield! {
    pub struct MultibootInfoFlags(u32);
    impl Debug;

    mem_info, _: 0;
    boot_device, _: 1;
    cmdline, _: 2;
    mods, _: 3;
    mmap, _: 6;
}

#[derive(Debug)]
#[repr(C)]
pub struct MultibootInfo {
    flags: MultibootInfoFlags,
    mem_info: MultibootMemInfo,
    _boot_device: u32,
    cmdline: *const u8,
    mods_count: u32,
    mods_addr: *const MultibootMod,
    // TODO
    _elf_1: u32,
    _elf_2: u32,
    _elf_3: u32,
    _elf_4: u32,
    mmap_length: u32,
    mmap_addr: *const MultibootMmap,
}

impl MultibootInfo {
    pub fn mem_info(&self) -> Option<&MultibootMemInfo> {
        if self.flags.mem_info() {
            Some(&self.mem_info)
        } else {
            None
        }
    }

    /// Returns the command line in unchecked string.
    /// I assume most bootloaders only support ascii anyway...
    pub fn cmdline(&self) -> Option<&str> {
        if self.flags.cmdline() {
            let str_slice = unsafe { slice::from_raw_parts(self.cmdline, strlen(self.cmdline)) };
            Some(unsafe { ::core::str::from_utf8_unchecked(str_slice) })
        } else {
            None
        }
    }

    /// Returns the addresses of the mods as u32
    pub fn mods(&self) -> Option<&[MultibootMod]> {
        if self.flags.mods() {
            Some(unsafe { slice::from_raw_parts(self.mods_addr, self.mods_count as usize) })
        } else {
            None
        }
    }

    pub fn mmap(&self) -> impl Iterator<Item = &MultibootMmap> {
        if self.flags.mmap() {
            MultibootMmapIterator::new(self.mmap_addr, self.mmap_length)
        } else {
            MultibootMmapIterator::new(::core::ptr::null(), 0)
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct MultibootMemInfo {
    pub mem_lower: u32,
    pub mem_upper: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct MultibootMod {
    pub mod_start: u32,
    pub mod_end: u32,
    string: *const u8,
    _reserved: u32,
}

impl MultibootMod {
    pub fn string(&self) -> Option<&str> {
        if self.string.is_null() {
            None
        } else {
            let str_slice = unsafe { slice::from_raw_parts(self.string, strlen(self.string)) };
            Some(unsafe { ::core::str::from_utf8_unchecked(str_slice) })
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct MultibootMmap {
    size: u32,
    pub base_addr: u64,
    pub length: u64,
    typ: u32,
}

impl MultibootMmap {
    pub fn is_available(&self) -> bool {
        self.typ == 1
    }
}

struct MultibootMmapIterator<'m> {
    current: u32,
    end: u32,
    _phantom: PhantomData<&'m MultibootMmap>,
}

impl<'m> MultibootMmapIterator<'m> {
    fn new(mmap_addr: *const MultibootMmap, mmap_length: u32) -> Self {
        MultibootMmapIterator {
            current: mmap_addr as u32,
            end: (mmap_addr as u32) + mmap_length,
            _phantom: PhantomData::default(),
        }
    }
}

impl<'m> Iterator for MultibootMmapIterator<'m> {
    type Item = &'m MultibootMmap;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let tmp = unsafe { &*(self.current as *const MultibootMmap) };
            self.current += tmp.size + 4;
            Some(tmp)
        } else {
            None
        }
    }
}

unsafe fn strlen(c: *const u8) -> usize {
    let mut len: usize = 0;
    while *c.add(len) != 0 {
        len += 1;
    }

    len
}
