use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevMemError {
    #[error("Failed to open /dev/mem: {0}")]
    OpenError(#[from] std::io::Error),

    #[error("Memory mapping failed")]
    MmapError,

    #[error("Invalid alignment: address {address:#x} must be aligned to {size} bytes")]
    AlignmentError { address: u64, size: usize },

    #[error("Invalid access size: {0}")]
    InvalidSize(usize),
}

pub type Result<T> = std::result::Result<T, DevMemError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessWidth {
    Byte = 1,
    Word = 2,
    Long = 4,
    LongLong = 8,
}

impl AccessWidth {
    pub fn size(&self) -> usize {
        *self as usize
    }

    pub fn from_size(size: usize) -> Result<Self> {
        match size {
            1 => Ok(AccessWidth::Byte),
            2 => Ok(AccessWidth::Word),
            4 => Ok(AccessWidth::Long),
            8 => Ok(AccessWidth::LongLong),
            _ => Err(DevMemError::InvalidSize(size)),
        }
    }
}

pub struct DevMem {
    file: File,
}

impl DevMem {
    pub fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open("/dev/mem")?;

        Ok(DevMem { file })
    }

    fn check_alignment(&self, address: u64, width: AccessWidth) -> Result<()> {
        let size = width.size();
        if address % size as u64 != 0 {
            return Err(DevMemError::AlignmentError { address, size });
        }
        Ok(())
    }

    pub fn read(&self, address: u64, width: AccessWidth) -> Result<u64> {
        self.check_alignment(address, width)?;

        let page_size = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) } as usize;
        let page_offset = (address as usize) & (page_size - 1);
        let map_base = address - page_offset as u64;
        let map_size = page_offset + width.size();

        unsafe {
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                map_size,
                libc::PROT_READ,
                libc::MAP_SHARED,
                self.file.as_raw_fd(),
                map_base as libc::off_t,
            );

            if ptr == libc::MAP_FAILED {
                return Err(DevMemError::MmapError);
            }

            let virt_addr = (ptr as usize + page_offset) as *const u8;

            let value = match width {
                AccessWidth::Byte => *(virt_addr as *const u8) as u64,
                AccessWidth::Word => *(virt_addr as *const u16) as u64,
                AccessWidth::Long => *(virt_addr as *const u32) as u64,
                AccessWidth::LongLong => *(virt_addr as *const u64),
            };

            libc::munmap(ptr, map_size);

            Ok(value)
        }
    }

    pub fn write(&self, address: u64, value: u64, width: AccessWidth) -> Result<()> {
        self.check_alignment(address, width)?;

        let page_size = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) } as usize;
        let page_offset = (address as usize) & (page_size - 1);
        let map_base = address - page_offset as u64;
        let map_size = page_offset + width.size();

        unsafe {
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                map_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                self.file.as_raw_fd(),
                map_base as libc::off_t,
            );

            if ptr == libc::MAP_FAILED {
                return Err(DevMemError::MmapError);
            }

            let virt_addr = (ptr as usize + page_offset) as *mut u8;

            match width {
                AccessWidth::Byte => *(virt_addr as *mut u8) = value as u8,
                AccessWidth::Word => *(virt_addr as *mut u16) = value as u16,
                AccessWidth::Long => *(virt_addr as *mut u32) = value as u32,
                AccessWidth::LongLong => *(virt_addr as *mut u64) = value,
            }

            libc::munmap(ptr, map_size);

            Ok(())
        }
    }

    pub fn read_modify_write(&self, address: u64, value: u64, width: AccessWidth) -> Result<u64> {
        let old_value = self.read(address, width)?;
        self.write(address, value, width)?;
        Ok(old_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_width() {
        assert_eq!(AccessWidth::Byte.size(), 1);
        assert_eq!(AccessWidth::Word.size(), 2);
        assert_eq!(AccessWidth::Long.size(), 4);
        assert_eq!(AccessWidth::LongLong.size(), 8);
    }

    #[test]
    fn test_from_size() {
        assert_eq!(AccessWidth::from_size(1).unwrap(), AccessWidth::Byte);
        assert_eq!(AccessWidth::from_size(2).unwrap(), AccessWidth::Word);
        assert_eq!(AccessWidth::from_size(4).unwrap(), AccessWidth::Long);
        assert_eq!(AccessWidth::from_size(8).unwrap(), AccessWidth::LongLong);
        assert!(AccessWidth::from_size(3).is_err());
    }
}
