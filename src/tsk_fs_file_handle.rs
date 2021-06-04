#![allow(non_camel_case_types)]

use std::io::{Read, Seek, SeekFrom};
use std::ffi::CStr;
use std::convert::TryInto;
use super::{
    tsk_fs_file::TskFsFile,
    tsk_fs_attr::TskFsAttr,
    errors::TskError,
    bindings as tsk
};


/// TskFsFileHandle struct is another entry point to read file data.
/// 'f -> TskFsFileHandle can never last longer than the file
/// 'fs -> TskFsFileHandle can never last longer than the file system
/// 
pub struct TskFsFileHandle<'f, 'fs>{
    /// The TskFsFile that is being used
    tsk_fs_file: &'f TskFsFile<'fs>,
    /// The TskFsAttr that contains the information
    /// for write operations
    tsk_fs_attr: TskFsAttr<'fs, 'f>,
    /// The read pointer
    _offset: i64,
    /// The read flags
    read_flag: tsk::TSK_FS_FILE_READ_FLAG_ENUM
}

impl<'f, 'fs> TskFsFileHandle<'f, 'fs> {
    /// Create TskFsFileHandle from TskFsFile, TskFsAttr and read flag.
    pub fn new(
        tsk_fs_file: &'f TskFsFile<'fs>,
        tsk_fs_attr: TskFsAttr<'fs, 'f>,
        read_flag: tsk::TSK_FS_FILE_READ_FLAG_ENUM
    ) -> Result<Self,TskError> {
        Ok( Self {
            tsk_fs_file,
            tsk_fs_attr,
            _offset: 0,
            read_flag
        })
    }
}

impl<'f, 'fs> Read for TskFsFileHandle<'f, 'fs> {
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
            buf.len().try_into().unwrap(),
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
                    self._offset += bytes_read as i64;
                    return Ok(bytes_read as usize);
                }
        };
    }
}

impl<'f, 'fs> Seek for TskFsFileHandle<'f, 'fs> {
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