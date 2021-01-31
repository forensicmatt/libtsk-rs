use std::ffi::CStr;
use crate::{
    errors::TskError,
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

    /// TskFsName represents a directory file
    pub fn is_dir(&self) -> bool {
        let type_ = unsafe {(*self.0).type_};
        type_ == tsk::TSK_FS_NAME_TYPE_ENUM_TSK_FS_NAME_TYPE_DIR
    }

    /// Get the inode for this TSK_FS_NAME
    pub fn get_inode(&self) -> u64 {
        unsafe {(*self.0).meta_addr}
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

    /// Get the short name of the attribute if available
    pub fn shrt_name(&self) -> Option<String> {
        // First check if the name is null
        if unsafe { (*self.0).shrt_name }.is_null() {
            return None;
        }
        let shrt_name = unsafe { CStr::from_ptr((*self.0).shrt_name) }.to_string_lossy();
        Some(shrt_name.to_string().clone())
    }
}
impl std::fmt::Debug for TskFsName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsName")
         .field("name", &self.name())
         .field("shrt_name", &self.shrt_name())
         .field("meta_addr", &unsafe{(*self.0).meta_addr})
         .field("meta_seq", &unsafe{(*self.0).meta_seq})
         .field("par_addr", &unsafe{(*self.0).par_addr})
         .field("par_seq", &unsafe{(*self.0).par_seq})
         .field("type", &unsafe{(*self.0).type_})
         .field("flags", &unsafe{(*self.0).flags})
         .field("date_added", &unsafe{(*self.0).date_added})
         .finish()
    }
}