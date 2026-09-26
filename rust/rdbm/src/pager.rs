//! Positioned reads and writes of whole pages in the database file.
//!
//! Every call goes straight to the operating system; there is no cache.

use std::fs::File;
use std::io;
use std::os::unix::fs::FileExt;

#[derive(Debug)]
pub struct Pager {
    file: File,
    page_size: u64,
    /// Tests set this to simulate a crash: once it reaches zero, every
    /// write fails, as though the process had died.
    #[cfg(test)]
    pub writes_left: std::cell::Cell<Option<usize>>,
}

impl Pager {
    pub fn new(file: File, page_size: u32) -> Self {
        Self {
            file,
            page_size: u64::from(page_size),
            #[cfg(test)]
            writes_left: std::cell::Cell::new(None),
        }
    }

    pub fn file_len(&self) -> io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }

    /// Reads `count` consecutive pages starting at page `first`.
    pub fn read_pages(&self, first: u64, count: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; (count * self.page_size) as usize];
        self.file.read_exact_at(&mut buf, first * self.page_size)?;
        Ok(buf)
    }

    pub fn read_page(&self, page: u64) -> io::Result<Vec<u8>> {
        self.read_pages(page, 1)
    }

    /// Writes `bytes` starting at byte `offset` within page `page`.
    pub fn write_at(&self, page: u64, offset: u64, bytes: &[u8]) -> io::Result<()> {
        #[cfg(test)]
        if let Some(n) = self.writes_left.get() {
            if n == 0 {
                return Err(io::Error::other("simulated crash"));
            }
            self.writes_left.set(Some(n - 1));
        }
        self.file
            .write_all_at(bytes, page * self.page_size + offset)
    }

    pub fn write_page(&self, page: u64, bytes: &[u8]) -> io::Result<()> {
        self.write_at(page, 0, bytes)
    }

    /// Flushes written data to stable storage.
    pub fn sync(&self) -> io::Result<()> {
        self.file.sync_data()
    }
}
