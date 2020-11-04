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
    handle: NonNull<tsk::TSK_FS_FILE>
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
}
impl Drop for TskFsFile {
    fn drop(&mut self) {
        unsafe { tsk::tsk_fs_file_close(self.handle.as_ptr()) };
    }
}