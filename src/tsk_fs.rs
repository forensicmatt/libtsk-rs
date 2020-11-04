use std::ptr::NonNull;
use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_img::TskImg,
    tsk_fs_file::TskFsFile,
    bindings as tsk
};


/// Wrapper for TSK_FS_INFO 
#[derive(Debug)]
pub struct TskFs {
    /// The ptr to the TSK_FS_INFO struct
    pub handle: NonNull<tsk::TSK_FS_INFO>
}
impl TskFs {
    /// Create a TSK_FS_INFO wrapper given the TskImg and offset of the file system
    pub fn new(tsk_img: &TskImg, offset: u64) -> Result<TskFs, TskError> {
        // Get a pointer to the TSK_FS_INFO sturct
        let tsk_fs = unsafe {tsk::tsk_fs_open_img(
            tsk_img.handle.as_ptr(),
            offset as i64 as _,
            0
        )};

        // Ensure that the ptr is not null
        let handle = match NonNull::new(tsk_fs) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { tsk::tsk_error_get() };
                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!("There was an error opening the fs handle at offset {}: {}", offset, error_msg)
                ));
            },
            Some(h) => h
        };

        Ok( Self { handle } )
    }

    /// Open a file by a given path. (use '/' as separators)
    pub fn file_open(&self, path: &str) -> Result<TskFsFile, TskError> {
        TskFsFile::from_path(&self, path)
    }

    /// Open a file by a given inode.
    pub fn file_open_meta(&self, inode: u64) -> Result<TskFsFile, TskError> {
        TskFsFile::from_meta(&self, inode)
    }
}
impl Drop for TskFs {
    fn drop(&mut self) {
        unsafe { tsk::tsk_fs_close(self.handle.as_ptr()) };
    }
}