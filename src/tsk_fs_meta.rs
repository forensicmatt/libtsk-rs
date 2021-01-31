use std::ffi::CStr;
use crate::{
    errors::TskError,
    bindings as tsk
};


/// Wrapper for TSK_FS_META
pub struct TskFsMeta(*const tsk::TSK_FS_META);
impl TskFsMeta {
    /// Get TskFsMeta wrapper given a TSK_FS_META pointer
    pub fn from_ptr(tsk_fs_meta: *const tsk::TSK_FS_META) -> Result<Self, TskError> {
        if tsk_fs_meta.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::tsk_fs_meta_error(
                format!("Error TSK_FS_META is null: {}", error_msg)
            ));
        }

        Ok(Self(tsk_fs_meta))
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

    /// Get the inode
    pub fn addr(&self) -> u64 {
        unsafe { (*self.0).addr }
    }

    /// Get type
    pub fn meta_type(&self) -> tsk::TSK_FS_META_TYPE_ENUM {
        unsafe { (*self.0).type_ }
    }

    /// Get flags
    pub fn flags(&self) -> Vec<tsk::TSK_FS_META_FLAG_ENUM> {
        let mut flags = vec![];
        match unsafe { (*self.0).flags } {
            f if f as i32 & tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_ALLOC as i32 > 0 => flags.push(tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_ALLOC),
            f if f as i32 & tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_UNALLOC as i32 > 0 => flags.push(tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_UNALLOC),
            f if f as i32 & tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_USED as i32 > 0 => flags.push(tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_USED),
            f if f as i32 & tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_UNUSED as i32 > 0 => flags.push(tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_UNUSED),
            f if f as i32 & tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_COMP as i32 > 0 => flags.push(tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_COMP),
            f if f as i32 & tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_ORPHAN as i32 > 0 => flags.push(tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_ORPHAN),
            _ => {}
        }
        flags
    }

    /// Allocation check
    pub fn is_unallocated(&self) -> bool {
        self.flags().iter().any(|f| match f {
            tsk::TSK_FS_META_FLAG_ENUM::TSK_FS_META_FLAG_UNALLOC => true,
            _ => false
        })
    }

}

impl std::fmt::Debug for TskFsMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsMeta")
            .field("size", &self.size())
            .field("crtime", &self.crtime())
            .field("mtime", &self.mtime())
            .field("atime", &self.atime())
            .field("addr", &self.addr())
            .field("type", &self.meta_type())
            .field("flags", &self.flags())
            .finish()
    }
}