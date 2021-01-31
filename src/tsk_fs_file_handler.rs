#![allow(non_camel_case_types)]

use std::io::{Read, Seek, SeekFrom};
use std::ffi::CStr;
use super::{
    tsk_fs_file::TskFsFile,
    tsk_fs_attr::TskFsAttr,
    errors::TskError,
    bindings as tsk
};

/// TskFsFileHandle struct is used to read file data.
pub struct TskFsFileHandle<'fs>{
    tsk_fs_file: &'fs TskFsFile<'fs>,
    tsk_fs_attr: TskFsAttr<'fs, 'fs>,
    _offset: i64,
    read_flag: tsk::TSK_FS_FILE_READ_FLAG_ENUM
}

impl<'fs> TskFsFileHandle<'fs> {
    /// Create TskFsFileHandle from TskFsFile, TskFsAttr and read flag.
    pub fn new(tsk_fs_file: &'fs TskFsFile<'fs>, tsk_fs_attr: TskFsAttr<'fs, 'fs>, read_flag: tsk::TSK_FS_FILE_READ_FLAG_ENUM) -> Result<Self,TskError> {
        Ok( Self {
            tsk_fs_file,
            _offset: 0,
            tsk_fs_attr,
            read_flag
        })
    }
}

impl<'fs> Read for TskFsFileHandle<'fs> {
    fn read(&mut self, buf: &mut [u8]) ->  std::io::Result<usize> {
        if self._offset == self.tsk_fs_attr.size() {
            return Ok(0);
        }
        let bytes_read = unsafe{tsk::tsk_fs_file_read_type(
            self.tsk_fs_file.into(),
            self.tsk_fs_attr.attr_type(),
            self.tsk_fs_attr.id(),
            self._offset,
            buf.as_mut_ptr() as *mut i8,
            buf.len() as u64,
            self.read_flag
        )};
        match bytes_read {
            -1 => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { tsk::tsk_error_get() };
                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
                return Err(
                    std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "TskError : {}",error_msg 
                    )
                ));
            }
            _ => {
                    self._offset+=bytes_read;
                    return Ok(bytes_read as usize);
                }
        };
    }
}

impl<'fs> Seek for TskFsFileHandle<'fs> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64>{
        match pos {
            SeekFrom::Start(o) => {
                if o > self.tsk_fs_attr.size() as u64{
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Start({}) is greater than attr size {}", o, self.tsk_fs_attr.size()
                        )
                    ));
                }
                else {
                    self._offset = o as i64;
                    return Ok(o);
                }
            },
            SeekFrom::Current(o) => {
                let new_offset = self._offset + o;
                if  new_offset > self.tsk_fs_attr.size() {
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Current({}) is greater than attr size. new_offset = {} + {} = {}, self.tsk_fs_attr.size() = {}", o, self._offset, o, new_offset, self.tsk_fs_attr.size()
                        )
                    ));
                }
                else if new_offset < 0 {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Cannot seek Current({}) from offset {}", o, self._offset)
                        )
                    );
                }
                else {
                    self._offset = new_offset;
                }
            },
            SeekFrom::End(o) => {
                let new_offset = self.tsk_fs_attr.size() + o;
                if  new_offset > self.tsk_fs_attr.size() {
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Current({}) is greater than attr size. new_offset = {} + {} = {}, self.tsk_fs_attr.size() = {}", o, self._offset, o, new_offset, self.tsk_fs_attr.size()
                        )
                    ));
                }
                else if new_offset < 0 {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Cannot seek Current({}) from offset {}", o, self._offset)
                        )
                    );
                }
                else {
                    self._offset = new_offset;
                }
            }
        }

        Ok(self._offset as u64)
    }
}