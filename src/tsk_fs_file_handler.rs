#![allow(non_camel_case_types)]

use std::io::{Read, Seek, SeekFrom};
use std::ffi::{CStr, CString};
use super::{
    tsk_fs_file::TskFsFile,
    tsk_fs_attr::TskFsAttrType,
    errors::TskError,
    bindings as tsk
};

#[derive(Copy, Clone)]
pub enum TskFsFileReadFlag {
    None,
    Slack,
    NoID
}

#[derive(Copy, Clone)]
pub struct TskFsFileHandler<'fs>{
    tsk_fs_file: &'fs TskFsFile<'fs>,
    _offset: i64,
    attr_size: i64,
    id: u16,
    attr_to_read: TskFsAttrType,
    pub read_flag: TskFsFileReadFlag
}

/// TskFsFileHandler struct is used to read file data.
impl<'fs> TskFsFileHandler<'fs> {
    /// Create TskFsFileHandler from TskFsFile. This will default to read $DATA attribute.
    pub fn new(tsk_fs_file: &'fs TskFsFile<'fs>) -> Result<Self,TskError> {
        let attr_size = (&tsk_fs_file.get_meta()?).size();
        let mut attr_to_read: TskFsAttrType = TskFsAttrType::NTFS_DATA;
        Ok( Self {
            tsk_fs_file,
            _offset: 0,
            attr_size,
            attr_to_read,
            read_flag: TskFsFileReadFlag::NoID, // Ignore the id and read $DATA
            id: 1
        })
    }
    /// Create TskFsFileHandler from TskFsFile and an attribute id.
    /// You can call `get_attr` function on TskFsFile which will return `TskFsAttr` then call `get_attr_iter` function
    /// which will return `TskFsAttrIterator` struct. Then iterate over `TskFsAttrIterator` to read all available attributes.
    /// Pass the TskFsFile struct along with the id of the attribute you want to read.
    pub fn new_with_id(tsk_fs_file: &'fs TskFsFile<'fs>, id: u16) -> Result<Self, TskError> {
        let attr_size = (&tsk_fs_file.get_meta()?).size();
        let mut attr_to_read: TskFsAttrType = TskFsAttrType::NTFS_DATA;
        for attr in tsk_fs_file.get_attr_iter().expect("Could not get attribute iterator for $MFT"){
            let attr_id = unsafe{*attr.tsk_fs_attr}.id;
            let attr_type = unsafe{*attr.tsk_fs_attr}.type_;
            if id == attr_id {
                attr_to_read = TskFsAttrType::from(attr_type);
            }
        }
        Ok( Self {
            tsk_fs_file,
            _offset: 0,
            attr_size,
            attr_to_read,
            read_flag: TskFsFileReadFlag::Slack,
            id
        })
    }
}

impl<'fs> Read for TskFsFileHandler<'fs> {
    fn read(&mut self, buf: &mut [u8]) ->  std::io::Result<usize> {
        let bytes_read = unsafe{tsk::tsk_fs_file_read_type(
            self.tsk_fs_file.into(),
            self.attr_to_read as i32,
            self.id,
            self._offset as i64,
            buf.as_mut_ptr() as *mut i8,
            buf.len() as u64,
            self.read_flag as i32
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

impl<'fs> Seek for TskFsFileHandler<'fs> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64>{
        match pos {
            SeekFrom::Start(o) => {
                if o > self.attr_size as u64{
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Start({}) is greater than attr size {}", o, self.attr_size
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
                if  new_offset > self.attr_size {
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Current({}) is greater than attr size. new_offset = {} + {} = {}, self.attr_size = {}", o, self._offset, o, new_offset, self.attr_size
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
                let new_offset = self.attr_size + o;
                if  new_offset > self.attr_size {
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Current({}) is greater than attr size. new_offset = {} + {} = {}, self.attr_size = {}", o, self._offset, o, new_offset, self.attr_size
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