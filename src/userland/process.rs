use alloc::boxed::Box;
use alloc::vec::Vec;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::kfs::FileHandle;

lazy_static! {
    pub static ref USER_PROCESS: Mutex<Process> = Mutex::new(Process::new());
}

pub struct Process {
    file_descriptors: [Option<Box<dyn FileHandle + Sync + Send + 'static>>; Process::MAX_FD],
    pub memory: Vec<u8>,
    pub brk: usize,
}

impl Process {
    const MAX_FD: usize = 32;
    const MEMORY_SIZE: usize = 1024 * 10;

    pub fn new() -> Process {
        let mut process = Process {
            file_descriptors: [
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
            ],
            memory: Vec::with_capacity(Self::MEMORY_SIZE),
            brk: 0,
        };
        process.memory.push(0);
        process
    }

    /// Return an error of no file descriptor is available
    pub fn store_file(&mut self, file: Box<dyn FileHandle + Sync + Send + 'static>) -> Result<u32, ()> {
        let (first_free_fd, option) = self.file_descriptors.iter_mut()
            .enumerate()
            .find(|(_i, o)| o.is_none())
            .ok_or(())?;

        option.replace(file);

        Ok(first_free_fd as u32)
    }

    pub fn get_file(&mut self, fd: u32) -> Option<&mut (dyn FileHandle + Sync + Send + 'static)> {
        use core::ops::DerefMut;
        let fd = fd as usize;

        if fd < self.file_descriptors.len() {
            self.file_descriptors[fd]
                .as_mut()
                .map(|b| b.deref_mut())
        } else {
            None
        }
    }

    /// Remove the fd if any
    pub fn close_file(&mut self, fd: u32) -> Result<(), ()> {
        let fd = fd as usize;
        if fd < self.file_descriptors.len() {
            self.file_descriptors[fd as usize].take()
                .map(|_| ())
                .ok_or(())
        } else {
            Err(())
        }
    }
}

impl Default for Process {
    fn default() -> Self {
        Process::new()
    }
}
