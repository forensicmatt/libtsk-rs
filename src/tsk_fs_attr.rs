use std::io::{Read, Seek, SeekFrom};
use std::convert::{TryInto, From};
use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_fs_file::TskFsFile,
    bindings as tsk
};
use std::fmt::{Display, Formatter, Result as FmtReasult};

#[derive(Copy, Clone, Debug)]
pub enum TskFsAttrType {
    DEFAULT = 0x01,
    HFS_COMP_REC = 0x1103,
    HFS_DATA = 0x1100,
    HFS_EXT_ATTR = 0x1102,
    HFS_RSRC = 0x1101,
    NOT_FOUND = 0x00,
    NTFS_ATTRLIST = 0x20,
    NTFS_BITMAP = 0xB0,
    NTFS_DATA = 0x80,
    NTFS_EA = 0xE0,
    NTFS_EAINFO = 0xD0,
    NTFS_FNAME = 0x30,
    NTFS_IDXALLOC = 0xA0,
    NTFS_IDXROOT = 0x90,
    NTFS_LOG = 0x100,
    NTFS_OBJID_OR_VVER = 0x40,
    NTFS_PROP = 0xF0,
    NTFS_REPARSE_OR_SYMLNK = 0xC0,
    NTFS_SEC = 0x50,
    NTFS_SI = 0x10,
    NTFS_VINFO = 0x70,
    NTFS_VNAME = 0x60,
    UNIX_INDIR = 0x1001
}

impl From<i32> for TskFsAttrType {
    fn from(type_: i32) -> Self {
        match type_ {
            0x01 => TskFsAttrType::DEFAULT,
            0x1103 => TskFsAttrType::HFS_COMP_REC,
            0x1100 => TskFsAttrType::HFS_DATA,
            0x1102 => TskFsAttrType::HFS_EXT_ATTR,
            0x1101 => TskFsAttrType::HFS_RSRC,
            0x00 => TskFsAttrType::NOT_FOUND,
            0x20 => TskFsAttrType::NTFS_ATTRLIST,
            0xB0 => TskFsAttrType::NTFS_BITMAP,
            0x80 => TskFsAttrType::NTFS_DATA,
            0xE0 => TskFsAttrType::NTFS_EA,
            0xD0 => TskFsAttrType::NTFS_EAINFO,
            0x30 => TskFsAttrType::NTFS_FNAME,
            0xA0 => TskFsAttrType::NTFS_IDXALLOC,
            0x90 => TskFsAttrType::NTFS_IDXROOT,
            0x100 => TskFsAttrType::NTFS_LOG,
            0x40 => TskFsAttrType::NTFS_OBJID_OR_VVER,
            0xF0 => TskFsAttrType::NTFS_PROP,
            0xC0 => TskFsAttrType::NTFS_REPARSE_OR_SYMLNK,
            0x50 => TskFsAttrType::NTFS_SEC,
            0x10 => TskFsAttrType::NTFS_SI,
            0x70 => TskFsAttrType::NTFS_VINFO,
            0x60 => TskFsAttrType::NTFS_VNAME,
            0x1001 => TskFsAttrType::UNIX_INDIR,
            _ => TskFsAttrType::NOT_FOUND
        }
    }
}

impl Display for TskFsAttrType{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtReasult {
        write!(f, "{:?}", self)
    }
}

/// Wrapper for TSK_FS_ATTR. This maintains a lifetime reference of TskFsFile so
/// that we are guaranteed that the pointers are always valid. Otherwise, we
/// have no safety guarantee that the pointers are still available. 
/// 
/// `fs => Filesystem lifetime
/// 'f => File lifetime
pub struct TskFsAttr<'fs, 'f>{
    tsk_fs_file: &'f TskFsFile<'fs>,
    pub tsk_fs_attr: *const tsk::TSK_FS_ATTR,
    _offset: i64
}
impl<'fs, 'f> TskFsAttr<'fs, 'f> {
    /// Create a TSK_FS_ATTR wrapper given the TskFsFile and index of the attribute
    pub fn from_index(
        tsk_fs_file: &'f TskFsFile<'fs>, 
        tsk_fs_file_attr_get_idx: u16
    ) -> Result<Self, TskError> {
        // Get a pointer to the TSK_FS_ATTR sturct
        let tsk_fs_attr = unsafe {tsk::tsk_fs_file_attr_get_idx(
            tsk_fs_file.into(),
            tsk_fs_file_attr_get_idx as _
        )};

        // Check for error
        if tsk_fs_attr.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            return Err(
                TskError::tsk_attr_error(
                    format!(
                        "There was an error getting the TskFsAttr at index {}: {}", 
                        tsk_fs_file_attr_get_idx, error_msg
                    )
                )
            );
        }

        Ok(
            Self {
                tsk_fs_file, 
                tsk_fs_attr,
                _offset: 0
            }
        )
    }

    /// Create a TSK_FS_ATTR wrapper given the TskFsFile and index of the attribute
    pub fn from_default(
        tsk_fs_file: &'f TskFsFile<'fs>
    ) -> Result<Self, TskError> {
        // Get a pointer to the TSK_FS_ATTR sturct
        let tsk_fs_attr = unsafe {tsk::tsk_fs_file_attr_get(
            tsk_fs_file.into()
        )};

        // Check for error
        if tsk_fs_attr.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            return Err(
                TskError::tsk_attr_error(
                    format!(
                        "There was an error getting the default TskFsAttr: {}", 
                        error_msg
                    )
                )
            );
        }

        Ok(
            Self {
                tsk_fs_file, 
                tsk_fs_attr,
                _offset: 0
            }
        )
    }

    /// Get the name of the attribute if available
    pub fn name(&self) -> Option<String> {
        // First check if the name is null
        if unsafe { (*self.tsk_fs_attr).name }.is_null() {
            return None;
        }

        let name = unsafe { CStr::from_ptr((*self.tsk_fs_attr).name) }.to_string_lossy();
        Some(name.to_string().clone())
    }

    /// Get the size of this attribute
    pub fn size(&self) -> i64 {
        return unsafe { (*self.tsk_fs_attr).size }
    }

    /// Get a str representation of the type
    pub fn type_name(&self) -> TskFsAttrType {
        TskFsAttrType::from(unsafe { (*self.tsk_fs_attr).type_ })
    }
    /// Get an iterator based off this TskFsAttr struct
    pub fn into_iter(self) -> TskFsAttrIterator<'fs, 'f> {
        TskFsAttrIterator(self)
    }
}
impl<'fs, 'f> std::fmt::Debug for TskFsAttr<'fs, 'f> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsAttr")
         .field("id", &(unsafe{*self.tsk_fs_attr}.id))
         .field("name", &self.name())
         .field("type", &(unsafe{*self.tsk_fs_attr}.type_))
         .field("type_name", &self.type_name())
         .field("size", &(unsafe{*self.tsk_fs_attr}.size))
         .finish()
    }
}
impl<'fs, 'f> Read for TskFsAttr<'fs, 'f> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let attr_size = self.size();

        let read_size = if buf.len() as u64 > attr_size as u64 {
            attr_size as u64
        } else {
            buf.len() as u64
        };

        // Get a pointer to the TSK_FS_FILE sturct
        let bytes_read = unsafe {tsk::tsk_fs_attr_read(
            self.tsk_fs_attr,
            self._offset,
            buf.as_mut_ptr() as _,
            read_size,
            tsk::TSK_FS_FILE_READ_FLAG_ENUM_TSK_FS_FILE_READ_FLAG_NONE
        )};

        if bytes_read == -1 {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("{}", error_msg)
                )
            );
        }
        // update offset by the number of bytes read
        self._offset += bytes_read;

        Ok(bytes_read as usize)
    }
}
impl<'fs, 'f> Seek for TskFsAttr<'fs, 'f> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let attr_size = self.size();

        match pos {
            SeekFrom::Start(o) => {
                if o > attr_size as u64 {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!(
                                "Offset Start({}) is greater than attribute size {}", 
                                o, 
                                attr_size
                            )
                        )
                    );
                } else {
                    self._offset = match o.try_into() {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(
                                std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Error casting offset to i64: {}", e)
                                )
                            );
                        }
                    }
                }
            },
            SeekFrom::Current(o) => {
                let location = o + self._offset;
                if location < 0 {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Cannot seek Current({}) from offset {}", o, self._offset)
                        )
                    );
                } else {
                    if location > attr_size {
                        return Err(
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!(
                                    "Offset Current({}) from {} is greater than attribute size {}", 
                                    o, self._offset, attr_size
                                )
                            )
                        );
                    } else {
                        self._offset = location;
                    }
                }
            },
            SeekFrom::End(o) => {
                let location = o + attr_size;
                if location < 0 {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Cannot seek End({}) from offset {}", o, self._offset)
                        )
                    );
                } else {
                    if location > attr_size {
                        return Err(
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!(
                                    "Offset Current({}) from {} is greater than attribute size {}", 
                                    o, self._offset, attr_size
                                )
                            )
                        );
                    } else {
                        self._offset = location;
                    }
                }
            }
        }

        Ok(self._offset as u64)
    }
}


/// An iterator over a TSK_FS_ATTR pointer which uses the
/// structs next attribute to iterate.
pub struct TskFsAttrIterator<'fs, 'f>(TskFsAttr<'fs, 'f>);
impl<'fs, 'f> Iterator for TskFsAttrIterator<'fs, 'f> {
    type Item = TskFsAttr<'fs, 'f>;
    
    fn next(&mut self) -> Option<TskFsAttr<'fs, 'f>> {
        if self.0.tsk_fs_attr.is_null() {
            return None;
        }

        let next = unsafe {
            TskFsAttr {
                tsk_fs_file: self.0.tsk_fs_file,
                tsk_fs_attr: (*self.0.tsk_fs_attr).next as *const tsk::TSK_FS_ATTR,
                _offset: 0
            }
        };

        let current = TskFsAttr {
            tsk_fs_file: self.0.tsk_fs_file,
            tsk_fs_attr: self.0.tsk_fs_attr,
            _offset: 0
        };

        self.0 = next;
        
        Some(current)
    }
}
