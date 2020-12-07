use std::ptr::NonNull;
use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_dir::TskFsDir,
    bindings as tsk
};


/// Wrapper for TSK_FS_NAME
pub struct TskFsName(*const tsk::TSK_FS_NAME);
impl TskFsName {
    pub fn from_ptr(tsk_fs_name: *const tsk::TSK_FS_NAME) -> Result<Self, TskError> {
        if tsk_fs_name.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::tsk_fs_name_error(
                format!("Error TSK_FS_NAME is null: {}", error_msg)
            ));
        }

        Ok(Self(tsk_fs_name))
    }

    /// Get the name of the attribute if available
    pub fn name(&self) -> Option<String> {
        // First check if the name is null
        if unsafe { (*self.0).name }.is_null() {
            return None;
        }
        let name = unsafe { CStr::from_ptr((*self.0).name) }.to_string_lossy();
        Some(name.to_string().clone())
    }
}
impl std::fmt::Debug for TskFsName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsName")
         .field("name", &self.name())
         .finish()
    }
}