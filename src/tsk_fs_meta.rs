use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_dir::TskFsDir,
    bindings as tsk
};


/// Wrapper for TSK_FS_META
pub struct TskFsMeta(*const tsk::TSK_FS_META);
impl TskFsMeta {
    pub fn from_ptr(TSK_FS_META: *const tsk::TSK_FS_META) -> Result<Self, TskError> {
        if TSK_FS_META.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::tsk_fs_meta_error(
                format!("Error TSK_FS_META is null: {}", error_msg)
            ));
        }

        Ok(Self(TSK_FS_META))
    }

    /// Get the size of the file
    pub fn size(&self) -> i64 {
        unsafe { (*self.0).size }
    }

    /// Get the creation time of the file (epoch time)
    pub fn crtime(&self) -> i64 {
        unsafe { (*self.0).crtime }
    }

    /// Get the modification time of the file (epoch time)
    pub fn mtime(&self) -> i64 {
        unsafe { (*self.0).mtime }
    }

    /// Get the access time of the file (epoch time)
    pub fn atime(&self) -> i64 {
        unsafe { (*self.0).atime }
    }

}

impl std::fmt::Debug for TskFsMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsMeta")
            .field("size", &self.size())
            .field("crtime", &self.crtime())
            .field("mtime", &self.mtime())
            .field("atime", &self.atime())
            .finish()
    }
}