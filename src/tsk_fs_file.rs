use std::ptr::{null_mut, NonNull};
use std::ffi::{CStr, CString};
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_meta::TskFsMeta,
    tsk_fs_attr::{TskFsAttr, TskFsAttrIterator},
    tsk_fs_file_handle::TskFsFileHandle,
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
            let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                .ok_or(
                    TskError::lib_tsk_error(
                        format!("There was an error opening {}. (no context)", path)
                    )
                )?;

            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
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
            let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                .ok_or(
                    TskError::lib_tsk_error(
                        format!("There was an error opening inode {} (no context)", inode)
                    )
                )?;

            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
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

    /// Return the number of attributes in the file. 
    pub fn attr_getsize(&self) -> Result<i32, TskError> {
        // Get a pointer to the TSK_FS_FILE sturct
        let attr_count = unsafe {tsk::tsk_fs_file_attr_getsize(
            self.into()
        )};

        if attr_count == -1 {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                .ok_or(
                    TskError::lib_tsk_error(
                        format!("There was an error getting attribute count for indoe {}. (no context)", self.get_meta().unwrap().addr())
                    )
                )?;

            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::lib_tsk_error(
                format!("There was an error getting attribute count for indoe {}: {}", self.get_meta().unwrap().addr(), error_msg)
            ));
        }

        Ok(attr_count)
    }

    /// Get the default TskFsAttr for this TskFsFile
    pub fn get_attr(&self) -> Result<TskFsAttr, TskError> {
        TskFsAttr::from_default(self)
    }

    /// Get the TskFsAttr at a given index for this TskFsFile (note this is not the id)
    pub fn get_attr_at_index(&self, index: u16) -> Result<TskFsAttr, TskError> {
        TskFsAttr::from_index(self, index)
    }

    /// Get a TskFsAttrIterator for this TskFsFile
    pub fn get_attr_iter<'f>(&'fs self) -> Result<TskFsAttrIterator<'f, 'fs>, TskError> {
        let tsk_fs_attr = TskFsAttr::from_index(self, 0)?;
        Ok(tsk_fs_attr.into_iter())
    }

    /// Get the TskFsMeta for this TskFsFile
    pub fn get_meta(&self) -> Result<TskFsMeta, TskError> {
        TskFsMeta::from_ptr(unsafe{(*self.tsk_fs_file_ptr).meta})
    }

    /// Get the TskFsFileHandle for this TskFsFile
    pub fn get_file_handle(
        &'fs self, 
        tsk_fs_attr: TskFsAttr<'fs, 'fs>, 
        read_flag: tsk::TSK_FS_FILE_READ_FLAG_ENUM
    ) -> Result<TskFsFileHandle, TskError> {
        TskFsFileHandle::new(self, tsk_fs_attr, read_flag)
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