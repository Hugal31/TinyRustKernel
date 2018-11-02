use no_std_io::*;

use super::DataBlock;

#[derive(Clone)]
pub(super) struct DataBlockReader<'a, I>
where
    I: Iterator<Item = &'a DataBlock> + Clone,
{
    /// Untouched iterator, represent the beginning of the datablocks
    begin: I,
    /// Block iterator
    iter: I,
    /// Current read block
    current: Option<&'a DataBlock>,
    /// Offset in current block
    current_offset: usize,
    /// Offset in the file
    offset: usize,
}

impl<'a, I> DataBlockReader<'a, I>
where
    I: Iterator<Item = &'a DataBlock> + Clone,
{
    /// Go to the next block
    fn next_block(&mut self) {
        if let Some(block) = self.current {
            self.offset += block.usage - self.current_offset;
            self.current_offset = 0;
            self.current = self.iter.next();
        }
    }

    fn go_to_end(&mut self) {
        while self.current.is_some() {
            self.next_block()
        }
    }

    fn seek_at(&mut self, offset: usize) -> Result<()> {
        if offset < self.offset {
            self.reset();
            return self.seek_at(offset);
        }

        while self.offset != offset {
            if let Some(block) = self.current {
                let remaining_offset = offset - self.offset;
                if (block.usage - self.current_offset) >= remaining_offset {
                    self.current_offset += remaining_offset;
                    self.offset += remaining_offset;
                } else {
                    self.next_block();
                }
            } else {
                // End of the file
                break;
            }
        }

        Ok(())
    }

    /// Go to the beginnig of the file
    fn reset(&mut self) {
        self.iter = self.begin.clone();
        self.current = self.iter.next();
        self.current_offset = 0;
        self.offset = 0;
    }
}

impl<'a, I> DataBlockReader<'a, I>
where
    I: Iterator<Item = &'a DataBlock> + Clone,
{
    pub fn new(mut iter: I) -> DataBlockReader<'a, I> {
        let begin = iter.clone();
        let current = iter.next();

        DataBlockReader {
            begin,
            iter,
            current,
            current_offset: 0,
            offset: 0,
        }
    }
}

impl<'a, I> Read for DataBlockReader<'a, I>
where
    I: Iterator<Item = &'a DataBlock> + Clone,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(block) = self.current {
            // There is still some blocks to read
            let size = block.read(buf, self.current_offset);
            self.offset += size;
            self.current_offset += size;
            // Check if we still have some data to read
            if size == buf.len() {
                Ok(size)
            } else {
                self.next_block();
                self.read(&mut buf[size..]).map(|s| size + s)
            }
        } else {
            // We are at the end of the file
            Ok(0)
        }
    }
}

impl<'a, I> Seek for DataBlockReader<'a, I>
where
    I: Iterator<Item = &'a DataBlock> + Clone,
{
    fn seek(&mut self, from: SeekFrom) -> Result<usize> {
        match from {
            SeekFrom::Start(offset) => self.seek_at(offset),
            SeekFrom::Current(offset) => self.seek_at((self.offset as isize + offset) as usize),
            SeekFrom::End(offset) => {
                self.go_to_end();
                self.seek_at((self.offset as isize + offset) as usize)
            }
        }.map(|()| self.offset)
    }
}
