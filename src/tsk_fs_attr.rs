use std::io::{Read, Seek, SeekFrom};
use std::convert::{TryInto};
use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_fs_file::TskFsFile,
    bindings as tsk
};

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
        unsafe { (*self.tsk_fs_attr).size }
    }

    /// Get the type of the attribute
    pub fn attr_type(&self) -> tsk::TSK_FS_ATTR_TYPE_ENUM {
        unsafe { (*self.tsk_fs_attr).type_ }
    }
    /// Get an iterator based off this TskFsAttr struct
    pub fn into_iter(self) -> TskFsAttrIterator<'fs, 'f> {
        TskFsAttrIterator(self)
    }

    /// Get the id of this attribute
    pub fn id(&self) -> u16 {
        unsafe { (*self.tsk_fs_attr).id as u16 }
    }
    
}
impl<'fs, 'f> std::fmt::Debug for TskFsAttr<'fs, 'f> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsAttr")
         .field("id", &self.id())
         .field("name", &self.name())
         .field("type", &self.attr_type())
         .field("size", &self.size())
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
            tsk::TSK_FS_FILE_READ_FLAG_ENUM::TSK_FS_FILE_READ_FLAG_NONE
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
