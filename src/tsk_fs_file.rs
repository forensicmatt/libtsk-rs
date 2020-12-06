use std::ptr::{null_mut, NonNull};
use std::ffi::{CStr, CString};
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    bindings as tsk
};


/// Wrapper for TSK_FS_FILE 
#[derive(Debug)]
pub struct TskFsFile {
    /// The ptr to the TSK_FS_FILE struct
    pub handle: NonNull<tsk::TSK_FS_FILE>
}
impl TskFsFile {
    /// Create a TSK_FS_FILE wrapper given TskFs and path
    pub fn from_path(tsk_fs: &TskFs, path: &str) -> Result<TskFsFile, TskError> {
        // Create a CString for the provided source
        let path_c = CString::new(path)
            .map_err(|e| TskError::generic(format!("Unable to create CString from path {}: {:?}", path, e)))?;

        // Get a pointer to the TSK_FS_FILE sturct
        let tsk_fs_file = unsafe {tsk::tsk_fs_file_open(
            tsk_fs.handle.as_ptr(),
            null_mut(),
            path_c.as_ptr() as _
        )};

        // Ensure that the ptr is not null
        let handle = match NonNull::new(tsk_fs_file) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { tsk::tsk_error_get() };
                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!("There was an error opening {}: {}", path, error_msg)
                ));
            },
            Some(h) => h
        };

        Ok( Self { handle } )
    }

    /// Create a TSK_FS_FILE wrapper given TskFs and inode
    pub fn from_meta(tsk_fs: &TskFs, inode: u64) -> Result<TskFsFile, TskError> {
        // Get a pointer to the TSK_FS_FILE sturct
        let tsk_fs_file = unsafe {tsk::tsk_fs_file_open_meta(
            tsk_fs.handle.as_ptr(),
            null_mut(),
            inode as _
        )};

        // Ensure that the ptr is not null
        let handle = match NonNull::new(tsk_fs_file) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { tsk::tsk_error_get() };
                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!("There was an error opening {}: {}", inode, error_msg)
                ));
            },
            Some(h) => h
        };

        Ok( Self { handle } )
    }

    /// Get the TskFsAttr at a given index for this TskFsFile (note this is not the id)
    pub fn get_attr_at_index(&self, index: u16) -> Result<TskFsAttr, TskError> {
        TskFsAttr::from_index(self, index)
    }

    /// Get a TskFsAttrIterator for this TskFsFile
    pub fn get_attr_iter(&self) -> Result<TskFsAttrIterator, TskError> {
        Ok(TskFsAttr::from_index(self, 0)?.into_iter())
    }
}
impl Drop for TskFsFile {
    fn drop(&mut self) {
        unsafe { tsk::tsk_fs_file_close(self.handle.as_ptr()) };
    }
}


/// Wrapper for TSK_FS_ATTR. This maintains a lifetime reference of TskFsFile so
/// that we are guaranteed that the pointers are always valid. Other wise we
/// have no safety guarantee that the pointers are still available. 
pub struct TskFsAttr<'a>{
    tsk_fs_file: &'a TskFsFile,
    tsk_fs_attr: *const tsk::TSK_FS_ATTR
}
impl<'a> TskFsAttr<'a> {
    /// Create a TSK_FS_ATTR wrapper given the TskFsFile and index of the attribute
    pub fn from_index(
        tsk_fs_file: &'a TskFsFile, 
        tsk_fs_file_attr_get_idx: u16
    ) -> Result<Self, TskError> {
        // Get a pointer to the TSK_FS_ATTR sturct
        let tsk_fs_attr = unsafe {tsk::tsk_fs_file_attr_get_idx(
            tsk_fs_file.handle.as_ptr(),
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
    pub fn into_iter(self) -> TskFsAttrIterator<'a> {
        TskFsAttrIterator(self)
    }
}
impl<'a> std::fmt::Debug for TskFsAttr<'a> {
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
pub struct TskFsAttrIterator<'a>(TskFsAttr<'a>);
impl<'a> Iterator for TskFsAttrIterator<'a> {
    type Item = TskFsAttr<'a>;
    
    fn next(&mut self) -> Option<TskFsAttr<'a>> {
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
