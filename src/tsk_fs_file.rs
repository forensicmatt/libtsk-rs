use std::ptr::{null_mut};
use std::ffi::{CStr, CString};
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_name::TskFsName,
    bindings as tsk
};


/// Wrapper for TSK_FS_FILE. The TSK_FS_FILE can never outlast the TSK_FS
/// 
/// 'fs => Filesystem lifetime.
#[derive(Debug)]
pub struct TskFsFile<'fs> {
    /// A TskFsFile can never outlive its TSK_FS_INFO
    tsk_fs_info_ptr: &'fs *mut tsk::TSK_FS_INFO,
    /// The ptr to the TSK_FS_FILE struct
    tsk_fs_file_ptr: *mut tsk::TSK_FS_FILE,
    /// We dont always want to free a file pointer
    _release: bool
}
impl<'fs> TskFsFile<'fs> {
    /// Create a TSK_FS_FILE wrapper given TskFs and path
    pub fn from_path(tsk_fs: &'fs TskFs, path: &str) -> Result<Self, TskError> {
        // Create a CString for the provided source
        let path_c = CString::new(path)
            .map_err(|e| TskError::generic(format!("Unable to create CString from path {}: {:?}", path, e)))?;

        // Get a pointer to the TSK_FS_FILE sturct
        let tsk_fs_file_ptr = unsafe {tsk::tsk_fs_file_open(
            tsk_fs.into(),
            null_mut(),
            path_c.as_ptr() as _
        )};

        if tsk_fs_file_ptr.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::lib_tsk_error(
                format!("There was an error opening {}: {}", path, error_msg)
            ));
        }

        Ok( Self { 
            tsk_fs_info_ptr: tsk_fs.into(),
            tsk_fs_file_ptr, 
            _release: true
        } )
    }

    /// Create a TSK_FS_FILE wrapper given TskFs and inode
    pub fn from_meta(tsk_fs: &'fs TskFs, inode: u64) -> Result<TskFsFile, TskError> {
        // Get a pointer to the TSK_FS_FILE sturct
        let tsk_fs_file_ptr = unsafe {tsk::tsk_fs_file_open_meta(
            tsk_fs.into(),
            null_mut(),
            inode as _
        )};

        if tsk_fs_file_ptr.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::lib_tsk_error(
                format!("There was an error opening inode {}: {}", inode, error_msg)
            ));
        }

        Ok( Self { 
            tsk_fs_info_ptr: tsk_fs.into(),
            tsk_fs_file_ptr, 
            _release: true
        } )
    }

    /// Get the TskFsAttr at a given index for this TskFsFile (note this is not the id)
    pub fn get_attr_at_index(&self, index: u16) -> Result<TskFsAttr, TskError> {
        TskFsAttr::from_index(self, index)
    }

    /// Get a TskFsAttrIterator for this TskFsFile
    pub fn get_attr_iter<'f>(&'fs self) -> Result<TskFsAttrIterator<'fs, 'f>, TskError> {
        let tsk_fs_attr = TskFsAttr::from_index(self, 0)?;
        Ok(tsk_fs_attr.into_iter())
    }

    /// Is unallocated
    pub fn is_unallocated(&self) -> bool {
        let meta = unsafe { (*self.tsk_fs_file_ptr).meta };
        unsafe{*meta}.flags & tsk::TSK_FS_META_FLAG_ENUM_TSK_FS_META_FLAG_UNALLOC > 0
    }

    /// Is Dir
    pub fn is_dir(&self) -> bool {
        let meta = unsafe { (*self.tsk_fs_file_ptr).meta };
        unsafe{*meta}.type_ & tsk::TSK_FS_META_TYPE_ENUM_TSK_FS_META_TYPE_DIR > 0
    }
}
impl<'fs> Into<*mut tsk::TSK_FS_FILE> for &TskFsFile<'fs> {
    fn into(self) -> *mut tsk::TSK_FS_FILE {
        self.tsk_fs_file_ptr
    }
}
impl<'fs> Drop for TskFsFile<'fs> {
    fn drop(&mut self) {
        if self._release {
            unsafe { tsk::tsk_fs_file_close(self.tsk_fs_file_ptr) };
        }
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
    tsk_fs_attr: *const tsk::TSK_FS_ATTR
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
                tsk_fs_attr
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

    /// Get a str representation of the type
    pub fn type_name(&self) -> &str {
        match unsafe { (*self.tsk_fs_attr).type_ } {
            0 => "NOT_FOUND",
            1 => "DEFAULT|HFS_DEFAULT",
            16 => "NTFS_SI",
            32 => "NTFS_ATTRLIST",
            48 => "NTFS_FNAME",
            64 => "NTFS_OBJID|NTFS_VVER",
            80 => "NTFS_SEC",
            96 => "NTFS_VNAME",
            112 => "NTFS_VINFO",
            128 => "NTFS_DATA",
            144 => "NTFS_IDXROOT",
            160 => "NTFS_IDXALLOC",
            176 => "NTFS_BITMAP",
            192 => "NTFS_SYMLNK|NTFS_REPARSE",
            208 => "NTFS_EAINFO",
            224 => "NTFS_EA",
            240 => "NTFS_PROP",
            256 => "NTFS_LOG",
            4097 => "UNIX_INDIR",
            4098 => "UNIX_EXTENT",
            4352 => "HFS_DATA|APFS_DATA",
            4353 => "HFS_RSRC|APFS_RSRC",
            4354 => "HFS_EXT_ATTR|APFS_EXT_ATTR",
            4355 => "HFS_COMP_REC|APFS_COMP_REC",
            _ => "<UNKNOWN>"
        }
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
                tsk_fs_attr: (*self.0.tsk_fs_attr).next as *const tsk::TSK_FS_ATTR
            }
        };

        let current = TskFsAttr {
            tsk_fs_file: self.0.tsk_fs_file,
            tsk_fs_attr: self.0.tsk_fs_attr
        };

        self.0 = next;
        
        Some(current)
    }
}
