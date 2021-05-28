use std::io::{Read, Seek, SeekFrom};
use std::convert::{TryInto};
use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_fs_file::TskFsFile,
    bindings as tsk
};


/// Wrapper for TSK_FS_ATTR_RUN pointer.
/// 
pub struct TskFsAttrRun<'dr, 'a, 'f, 'fs> {
    _nrd: &'dr NonResidentData<'a, 'f, 'fs>,
    tsk_fs_attr_run: *const tsk::TSK_FS_ATTR_RUN
}
impl<'dr, 'a, 'f, 'fs> TskFsAttrRun<'dr, 'a, 'f, 'fs> {
    /// Create a TskFsAttrRun from a TSK_FS_ATTR_RUN pointer and NonResidentData. The
    /// NonResidentData must last the life time of this TskFsAttrRun.
    pub fn from_nrd(
        _nrd: &'dr NonResidentData<'a, 'f, 'fs>,
        tsk_fs_attr_run: *const tsk::TSK_FS_ATTR_RUN
    ) -> Self {
        Self {
            _nrd,
            tsk_fs_attr_run
        }
    }

    /// Get the starting block address (in file system) of run
    /// 
    pub fn addr(&self) -> u64 {
        unsafe { (*self.tsk_fs_attr_run).addr }
    }

    /// Number of blocks in run (0 when entry is not in use) 
    /// 
    pub fn len(&self) -> u64 {
        unsafe { (*self.tsk_fs_attr_run).len }
    }

    /// Flags for run. 
    /// 
    pub fn flags(&self) -> i32 {
        unsafe { (*self.tsk_fs_attr_run).flags }
    }

    /// Flags for run. 
    /// 
    pub fn flags_str(&self) -> String {
        let mut string_vec = Vec::with_capacity(4);
        let flags = self.flags();

        if flags & tsk::TSK_FS_ATTR_RUN_FLAG_ENUM_TSK_FS_ATTR_RUN_FLAG_FILLER > 0 {
            string_vec.push("FILLER");
        }
        if flags & tsk::TSK_FS_ATTR_RUN_FLAG_ENUM_TSK_FS_ATTR_RUN_FLAG_SPARSE > 0 {
            string_vec.push("SPARSE");
        }
        if flags & tsk::TSK_FS_ATTR_RUN_FLAG_ENUM_TSK_FS_ATTR_RUN_FLAG_ENCRYPTED > 0 {
            string_vec.push("ENCRYPTED");
        }

        string_vec.join(" | ")
    }
}
impl <'dr, 'a, 'f, 'fs> std::fmt::Debug for TskFsAttrRun<'dr, 'a, 'f, 'fs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flags = format!("[0x{:04x}] {}", self.flags(), self.flags_str());
        f.debug_struct("TskFsAttrRun")
         .field("addr", &self.addr())
         .field("flags", &flags)
         .field("len", &self.len())
         .finish()
    }
}


/// An iterator over a TSK_FS_ATTR_RUN pointer which uses the
/// structs next attribute to iterate.
pub struct TskFsAttrRunIterator<'dr, 'a, 'f, 'fs>(TskFsAttrRun<'dr, 'a, 'f, 'fs>);
impl<'dr, 'a, 'f, 'fs> Iterator for TskFsAttrRunIterator<'dr, 'a, 'f, 'fs> {
    type Item = TskFsAttrRun<'dr, 'a, 'f, 'fs>;
    
    fn next(&mut self) -> Option<TskFsAttrRun<'dr, 'a, 'f, 'fs>> {
        if self.0.tsk_fs_attr_run.is_null() {
            return None;
        }

        let next = unsafe {
            TskFsAttrRun {
                _nrd: self.0._nrd,
                tsk_fs_attr_run: (*self.0.tsk_fs_attr_run).next as *const tsk::TSK_FS_ATTR_RUN
            }
        };

        let current = TskFsAttrRun {
            _nrd: self.0._nrd,
            tsk_fs_attr_run: self.0.tsk_fs_attr_run,
        };

        self.0 = next;
        
        Some(current)
    }
}


/// Wrapper for Non Resident Data
/// 
pub struct NonResidentData<'a, 'f, 'fs> {
    _tsk_fs_attr: &'a TskFsAttr<'f, 'fs>,
    nrd: tsk::TSK_FS_ATTR__bindgen_ty_1
}
impl <'a, 'f, 'fs> NonResidentData<'a, 'f, 'fs> {
    /// Create a NonResidentData from a TskFsAttr's lifetime and nrd struct (TSK_FS_ATTR__bindgen_ty_1)
    /// Thus, NonResidentData must live for the life time of the attribute it represents.
    /// 
    pub fn new(
        _tsk_fs_attr: &'a TskFsAttr<'f, 'fs>,
        nrd: tsk::TSK_FS_ATTR__bindgen_ty_1
    ) -> Self {
        Self {
            _tsk_fs_attr,
            nrd
        }
    }

    /// Number of initial bytes in run to skip before content begins. The size field does not include this length. 
    /// 
    pub fn skiplen(&self) -> u32 {
        self.nrd.skiplen
    }

    /// Number of bytes that are allocated in all clusters of non-resident run 
    /// (will be larger than size - does not include skiplen). This is defined when 
    /// the attribute is created and used to determine slack space.
    /// 
    pub fn allocsize(&self) -> i64 {
        self.nrd.allocsize
    }

    /// Number of bytes (starting from offset 0) that have data (including FILLER) 
    /// saved for them (smaller then or equal to size). This is defined when the attribute is created.
    /// 
    pub fn initsize(&self) -> i64 {
        self.nrd.initsize
    }

    /// Size of compression units (needed only if NTFS file is compressed)
    /// 
    pub fn compsize(&self) -> u32 {
        self.nrd.compsize
    }

    /// Get the first data run for this non resident data
    /// 
    pub fn run<'dr>(&'dr self) -> TskFsAttrRun<'dr, 'a, 'f, 'fs> {
        TskFsAttrRun::from_nrd(
            &self,
            self.nrd.run
        )
    }

    /// Get a TskFsAttrRunIterator based off this NonResidentData struct
    /// 
    pub fn iter<'dr>(&'dr self) -> TskFsAttrRunIterator<'dr, 'a, 'f, 'fs> {
        TskFsAttrRunIterator(self.run())
    }
}
impl <'a, 'f, 'fs> std::fmt::Debug for NonResidentData<'a, 'f, 'fs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonResidentData")
         .field("skiplen", &self.nrd.skiplen)
         .field("allocsize", &self.nrd.allocsize)
         .field("initsize", &self.nrd.initsize)
         .field("compsize", &self.nrd.compsize)
         .finish()
    }
}



/// Wrapper for TSK_FS_ATTR. This maintains a lifetime reference of TskFsFile so
/// that we are guaranteed that the pointers are always valid. Otherwise, we
/// have no safety guarantee that the pointers are still available. 
/// 
/// `fs => Filesystem lifetime
/// 'f => File lifetime
pub struct TskFsAttr<'f, 'fs>{
    tsk_fs_file: &'f TskFsFile<'fs>,
    pub tsk_fs_attr: *const tsk::TSK_FS_ATTR,
    _offset: i64
}
impl<'f, 'fs> TskFsAttr<'f, 'fs> {
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

    /// Get the flags of the attribute
    pub fn attr_flags(&self) -> tsk::TSK_FS_ATTR_FLAG_ENUM {
        unsafe { (*self.tsk_fs_attr).flags }
    }

    /// Get an iterator based off this TskFsAttr struct
    pub fn into_iter(self) -> TskFsAttrIterator<'fs, 'f> {
        TskFsAttrIterator(self)
    }

    /// Get the id of this attribute
    pub fn id(&self) -> u16 {
        unsafe { (*self.tsk_fs_attr).id as u16 }
    }
    
    /// Get the non-resident data or None if attribute is resident
    pub fn get_non_resident_data<'a>(&'a self) -> Option<NonResidentData<'a, 'f, 'fs>> {
        if (self.attr_flags() & tsk::TSK_FS_ATTR_FLAG_ENUM_TSK_FS_ATTR_NONRES) > 0 {
            unsafe { Some(
                NonResidentData::new(
                    &self,
                    (*self.tsk_fs_attr).nrd
                ) 
            )}
        } else {
            None
        }
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
