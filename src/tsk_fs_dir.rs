use std::ptr::{null_mut, NonNull};
use std::ffi::{CStr, CString};
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_name::TskFsName,
    bindings as tsk
};


/// Wrapper for TSK_FS_DIR
#[derive(Debug)]
pub struct TskFsDir<'fs> {
    /// A TskFsDir can never outlive its TskFs
    tsk_fs: &'fs TskFs,
    /// The ptr to the TSK_FS_DIR struct
    pub handle: NonNull<tsk::TSK_FS_DIR>
}
impl<'fs> TskFsDir<'fs> {
    /// Create a TSK_FS_DIR wrapper given TskFs and inode
    pub fn from_meta(tsk_fs: &'fs TskFs, inode: u64) -> Result<Self, TskError> {
        // Get a pointer to the TSK_FS_DIR sturct
        let tsk_fs_dir = unsafe {tsk::tsk_fs_dir_open_meta(
            tsk_fs.handle.as_ptr(),
            inode as _
        )};

        // Ensure that the ptr is not null
        let handle = match NonNull::new(tsk_fs_dir) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { tsk::tsk_error_get() };
                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!("There was an error opening {} as a dir: {}", inode, error_msg)
                ));
            },
            Some(h) => h
        };

        Ok( Self { tsk_fs, handle } )
    }

    pub fn get_name(&self, index: u64) -> Result<TskFsName, TskError> {
        // Get a pointer to the TSK_FS_FILE sturct
        let tsk_fs_name = unsafe {tsk::tsk_fs_dir_get_name(
            self.handle.as_ptr(),
            index as _
        )};

        if tsk_fs_name.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::tsk_fs_name_error(
                format!("Error getting TskFsName at index {} from TskFsDir {:?}: {}", index, &self, error_msg)
            ));
        }

        Ok(TskFsName::from_ptr(tsk_fs_name)?)
    }
}
impl<'fs> Drop for TskFsDir<'fs> {
    fn drop(&mut self) {
        unsafe { tsk::tsk_fs_dir_close(self.handle.as_ptr()) };
    }
}