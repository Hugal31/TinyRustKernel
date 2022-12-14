#![allow(safe_packed_borrows)]

mod reader;

use no_std_io::{Read, Seek};

use core::intrinsics::transmute;
use core::mem::{size_of, size_of_val};
use core::ptr::NonNull;
use core::slice;

use self::reader::DataBlockReader;

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

pub trait FileHandle: Read + Seek {}

static mut KFS: Kfs = Kfs::invalid();

pub unsafe fn init(start: u32, end: u32) -> Result<()> {
    let result = KFS.init(start, end);

    info!("KFS loaded between 0x{:X} and 0x{:X} ({} bytes)", start, end, end - start);
    result
}

pub fn get_fs() -> &'static Superblock {
    unsafe { KFS.superblock() }
}

#[derive(Debug)]
struct Kfs {
    superblock: Option<NonNull<Superblock>>,
}

#[allow(dead_code)]
impl Kfs {
    pub const fn invalid() -> Self {
        Kfs {
            superblock: None,
        }
    }

    /// Reference to the superblock and end address
    fn init(&mut self, start: u32, end: u32) -> Result<()> {
        let mem_size = (end - start) as usize;

        if mem_size < size_of::<Superblock>() {
            return Err(Error::MemTooSmall);
        }

        let superblock = Superblock::checked::<'static>(start as _)?;

        if mem_size < superblock.blk_cnt * BLK_SIZE {
            return Err(Error::MemTooSmall);
        }

        self.superblock = NonNull::new(unsafe { transmute(superblock as *const Superblock) } );

        self.validate()
    }

    fn superblock(&self) -> &'static Superblock {
        unsafe {
            &*self.superblock
                .expect("KFS was not initialized")
                .as_ptr()
        }
    }

    fn validate(&self) -> Result<()> {
        self.superblock().validate()?;

        Ok(())
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Superblock {
    magic: u32,
    name: [u8; NAME_SIZE],
    ctime: isize,
    blk_cnt: usize,
    inode_cnt: usize,
    inode_idx: usize,
    checksum: u32,
}

impl Superblock {
    fn checked<'a>(block_addr: *const Superblock) -> Result<&'a Superblock> {
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
        unsafe { crate::strings::cstr_to_str_unchecked(&self.name[0]) }
    }

    fn blocks(&self) -> &[Block] {
        unsafe {
            slice::from_raw_parts(
                transmute(self as *const Superblock),
                self.blk_cnt,
            )
        }
    }

    pub fn inodes(&self) -> impl Iterator<Item = &Inode> {
        InodeIterator::new(self)
    }

    pub fn reader<'a>(&'a self, inode: &'a Inode) -> impl FileHandle + Clone + 'a {
        DataBlockReader::new(inode.blocks(self))
    }

    fn validate(&self) -> Result<()> {
        self.inodes().map(|i| i.validate(&self)).collect()
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
    fn validate(&self, superblock: &Superblock) -> Result<()> {
        // Validate checksum
        let self_begin = unsafe {
            slice::from_raw_parts(
                transmute(self as *const Inode),
                size_of::<Inode>() - size_of_val(&self.checksum),
            )
        };

        if adler_checksum(self_begin) != self.checksum {
            return Err(Error::InvalidChecksum);
        }

        // Check if next inode and number of blk count are not out of bounds
        if self.d_blk_cnt >= MAX_DIRECT_BLK
            || self.i_blk_cnt >= MAX_INDIRECT_BLK
            || self.next_inode >= superblock.blk_cnt
        {
            return Err(Error::OutOfBounds);
        }

        // Check if direct and indirects blocks are out of bounds
        for &index in self
            .direct_blocks_idx()
            .iter()
            .chain(self.indirect_blocks_idx())
        {
            if index >= superblock.blk_cnt {
                return Err(Error::OutOfBounds);
            }
        }

        // TODO Check indirect blocks
        self.blocks(superblock).map(DataBlock::validate).collect()
    }

    fn direct_blocks_idx(&self) -> &[usize] {
        &self.d_blks[..self.d_blk_cnt]
    }

    fn indirect_blocks_idx(&self) -> &[usize] {
        &self.i_blks[..self.i_blk_cnt]
    }

    fn blocks<'a>(&'a self, superblock: &'a Superblock) -> DataBlockIterator<'a> {
        if self.i_blk_cnt != 0 {
            unimplemented!();
        }

        DataBlockIterator {
            superblock,
            ids: self.direct_blocks_idx(),
        }
    }

    pub fn filename(&self) -> &str {
        unsafe { crate::strings::cstr_to_str_unchecked(&self.filename[0]) }
    }
}

struct InodeIterator<'a> {
    superblock: &'a Superblock,
    idx: usize,
}

impl<'a> InodeIterator<'a> {
    fn new(superblock: &'a Superblock) -> Self {
        InodeIterator {
            superblock,
            idx: superblock.inode_idx,
        }
    }
}

impl<'a> Iterator for InodeIterator<'a> {
    type Item = &'a Inode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 || self.idx >= self.superblock.blk_cnt {
            return None;
        }

        let inode = unsafe { self.superblock.blocks()[self.idx].as_inode() };
        self.idx = inode.next_inode;
        Some(inode)
    }
}

struct DataBlock {
    index: u32,
    usage: usize,
    checksum: u32,
    data: [u8; BLK_SIZE - 3 * 4],
}

impl DataBlock {
    fn read(&self, buffer: &mut [u8], initial_cursor: usize) -> usize {
        use core::cmp::min;

        if initial_cursor >= self.usage {
            return 0;
        }

        let to_copy = min(self.usage - initial_cursor, buffer.len());
        buffer[..to_copy].copy_from_slice(&self.data[initial_cursor..initial_cursor + to_copy]);
        to_copy
    }

    fn validate(&self) -> Result<()> {
        let self_begin = unsafe {
            slice::from_raw_parts(
                transmute(self as *const DataBlock),
                size_of_val(&self.index) + size_of_val(&self.usage),
            )
        };

        // Introduce false checksum because the algorithm check on all the data,
        // and expect the checksum to be equal to 0.
        let false_checksum = [0; size_of::<u32>()];
        let checksum = adler_checksum(
            self_begin
                .iter()
                .chain(false_checksum.iter())
                .chain(self.data.iter()),
        );
        if checksum != self.checksum {
            Err(Error::InvalidChecksum)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
struct DataBlockIterator<'a> {
    superblock: &'a Superblock,
    ids: &'a [usize],
}

impl<'a> Iterator for DataBlockIterator<'a> {
    type Item = &'a DataBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ids.len() == 0 {
            None
        } else {
            let id = self.ids[0];
            let block = unsafe { self.superblock.blocks()[id].as_data() };
            self.ids = &self.ids[1..];
            Some(block)
        }
    }
}

fn adler_checksum<'a, I>(data: I) -> u32
where
    I: IntoIterator<Item = &'a u8>,
{
    const ALDER32_MOD: u32 = 65521;

    let mut a: u32 = 1;
    let mut b: u32 = 0;

    for &c in data {
        a = (a + c as u32) % ALDER32_MOD;
        b = (a + b) % ALDER32_MOD;
    }

    b << 16 | a
}
