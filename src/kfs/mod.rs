#![allow(safe_packed_borrows)]

use core::intrinsics::transmute;
use core::mem::{size_of, size_of_val};
use core::slice;

use crate::*; // TODO Remove

const MAGIC: u32 = 0xd35f9caa;
const NAME_SIZE: usize = 32;
const FNAME_SIZE: usize = 32;
const BLK_SIZE: usize = 4096;
const MAX_DIRECT_BLK: usize = 10;
const MAX_INDIRECT_BLK: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidChecksum,
    InvalidMagic,
    MemTooSmall,
    OutOfBounds,
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Kfs<'a> {
    superblock: &'a Superblock,
}

impl<'a> Kfs<'a> {
    /// Reference to the superblock and end address
    pub fn new(start: u32, end: u32) -> Result<Self> {
        let mem_size = (end - start) as usize;

        if mem_size < size_of::<Superblock>() {
            return Err(Error::MemTooSmall);
        }

        let superblock = Superblock::check::<'static>(start as _)?;

        if mem_size < superblock.blk_cnt * BLK_SIZE {
            return Err(Error::MemTooSmall);
        }

        Kfs {
            superblock,
        }.validate()
    }

    fn validate(self) -> Result<Self> {
        // TODO Iterate over inode and indirects to check all indexes
        Ok(self)
    }

    fn blocks(&self) -> &[Block] {
        unsafe { slice::from_raw_parts(
            transmute(self.superblock as *const Superblock),
            self.superblock.blk_cnt
        ) }
    }

    pub fn name(&self) -> &str {
        self.superblock.name()
    }

    pub fn inodes(&self) -> impl Iterator<Item = &Inode> {
        InodeIterator::new(self)
    }

    /// Return the size readed from the inode
    pub fn read(&self, inode: &Inode, buffer: &mut [u8], _initial_cursor: usize) -> usize {
        let mut cursor = 0;

        // TODO code block iterators
        if inode.direct_blocks_idx().len() != 1 {
            unimplemented!();
        }

        let block_idx = inode.direct_blocks_idx()[0];
        let block = unsafe { self.blocks()[block_idx].as_data() };
        block.read(buffer, 0)
    }
}

#[derive(Debug)]
#[repr(C, packed)]
struct Superblock {
    magic: u32,
    name: [u8; NAME_SIZE],
    pub ctime: isize,
    pub blk_cnt: usize,
    pub inode_cnt: usize,
    pub inode_idx: usize,
    checksum: u32,
}

impl Superblock {
    fn check<'a>(block_addr: *const Superblock) -> Result<&'a Superblock> {
        let block = unsafe { &*block_addr };
        if block.magic != MAGIC {
            return Err(Error::InvalidMagic);
        }

        let self_begin = unsafe {
            slice::from_raw_parts(
                transmute(block_addr),
                size_of::<Superblock>() - size_of_val(&block.checksum),
            )
        };

        if block.checksum != adler_checksum(self_begin) {
            return Err(Error::InvalidChecksum);
        }

        Ok(block)
    }

    pub fn name(&self) -> &str {
        unsafe { cstr_to_str_unchecked(&self.name[0]) }
    }
}

// Could use union, but non-copy union are unstable
struct Block([u8; BLK_SIZE]);

impl Block {
    unsafe fn as_inode(&self) -> &Inode {
        transmute(self)
    }

    unsafe fn as_data(&self) -> &DataBlock {
        transmute(self)
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Inode {
    number: i32,
    filename: [u8; FNAME_SIZE],
    size: usize,
    idx: usize,
    blk_count: usize,
    next_inode: usize,
    d_blk_cnt: usize,
    i_blk_cnt: usize,
    d_blks: [usize; MAX_DIRECT_BLK],
    i_blks: [usize; MAX_INDIRECT_BLK],
    checksum: u32,
}

impl Inode {
    fn checked<'a>(inode_addr: *const Inode) -> Result<&'a Inode> {
        let inode = unsafe { &*inode_addr };
        inode.check().map(|()| inode)
    }

    fn check<'a>(&self) -> Result<()> {
        let self_begin = unsafe {
            slice::from_raw_parts(
                transmute(self as *const Inode),
                size_of::<Inode>() - size_of_val(&self.checksum),
            )
        };

        if adler_checksum(self_begin) != self.checksum {
            return Err(Error::InvalidChecksum);
        }

        Ok(())
    }

    fn direct_blocks_idx(&self) -> &[usize] {
        unsafe { slice::from_raw_parts(&self.d_blks[0], self.d_blk_cnt) }
    }

    pub fn filename(&self) -> &str {
        unsafe { cstr_to_str_unchecked(&self.filename[0]) }
    }
}

struct InodeIterator<'a, 'k: 'a> {
    kfs: &'a Kfs<'k>,
    idx: usize,
}

impl<'a, 'k: 'a> InodeIterator<'a, 'k> {
    fn new(kfs: &'a Kfs<'k>) -> Self {
        InodeIterator {
            kfs: kfs,
            idx: kfs.superblock.inode_idx,
        }
    }
}

impl<'a, 'k: 'a> Iterator for InodeIterator<'a, 'k> {
    type Item = &'a Inode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 || self.idx >= self.kfs.superblock.blk_cnt {
            return None;
        }

        let inode = unsafe { self.kfs.blocks()[self.idx].as_inode() };
        self.idx = inode.next_inode;
        Some(inode)
    }
}

struct DataBlock {
    index: u32,
    usage: usize,
    checksum: usize,
    data: [u8; BLK_SIZE - 3 * 4],
}

impl DataBlock {
    fn read(&self, buffer: &mut [u8], initial_cursor: usize) -> usize {
        use core::cmp::min;

        if initial_cursor >= self.usage {
            return 0;
        }

        let to_copy = min(self.usage - initial_cursor, buffer.len());
        unsafe { (&self.data[0] as *const u8).copy_to_nonoverlapping(&mut buffer[0], to_copy) };
        to_copy
    }
}

fn adler_checksum(data: &[u8]) -> u32 {
    const ALDER32_MOD: u32 = 65521;

    let mut a: u32 = 1;
    let mut b: u32 = 0;

    for c in data.iter().cloned() {
        a = (a + c as u32) % ALDER32_MOD;
        b = (a + b) % ALDER32_MOD;
    }

    b << 16 | a
}

unsafe fn cstr_to_str_unchecked<'a>(c: *const u8) -> &'a str {
    let len = strlen(c);
    let s = slice::from_raw_parts(c, len);
    ::core::str::from_utf8_unchecked(s)
}

// TODO Centralize
unsafe fn strlen(c: *const u8) -> usize {
    let mut len: usize = 0;
    while *c.add(len) != 0 {
        len += 1;
    }

    len
}