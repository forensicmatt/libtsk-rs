use std::ptr::{null_mut, NonNull};
use std::ffi::{CStr, CString};
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_name::TskFsName,
    bindings as tsk
};


/// Wrapper for TSK_FS_DIR that implements helper functions.
pub struct TskFsDir<'fs> {
    /// A TskFsDir can never outlive its TskFs
    _tsk_fs: &'fs TskFs,
    /// The ptr to the TSK_FS_DIR struct
    pub handle: NonNull<tsk::TSK_FS_DIR>
}
impl<'fs> TskFsDir<'fs> {
    /// Create a TSK_FS_DIR wrapper given TskFs and inode
    pub fn from_meta(_tsk_fs: &'fs TskFs, inode: u64) -> Result<Self, TskError> {
        // Get a pointer to the TSK_FS_DIR sturct
        let tsk_fs_dir = unsafe {tsk::tsk_fs_dir_open_meta(
            _tsk_fs.handle.as_ptr(),
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

        Ok( Self { _tsk_fs, handle } )
    }

    /// Get a TskFsName at a given index of the TSK_FS_DIR
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

    /// Get an iterator that iterates TskFsNames of this TskFsDir
    pub fn get_name_iter<'d>(&'d self) -> DirNameIter<'fs, 'd> {
        DirNameIter {
            tsk_fs_dir: self,
            index: 0
        }
    }
}
impl<'fs> std::fmt::Debug for TskFsDir<'fs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tsk_fs_dir_ptr = self.handle.as_ptr();
        f.debug_struct("TskFsDir")
         .field("addr", unsafe{&(*tsk_fs_dir_ptr).addr})
         .field("seq", unsafe{&(*tsk_fs_dir_ptr).seq})
         .field("fs_file", unsafe{&(*tsk_fs_dir_ptr).fs_file})
         .field("names_used", unsafe{&(*tsk_fs_dir_ptr).names_used})
         .field("names_alloc", unsafe{&(*tsk_fs_dir_ptr).names_alloc})
         .field("names", unsafe{&(*tsk_fs_dir_ptr).names})
         .finish()
    }
}
impl<'fs> Drop for TskFsDir<'fs> {
    fn drop(&mut self) {
        unsafe { tsk::tsk_fs_dir_close(self.handle.as_ptr()) };
    }
}


/// DirNameIter is an iterator for the allocated TskFsName of a given TskFsDir.
pub struct DirNameIter<'fs, 'd>{
    tsk_fs_dir: &'d TskFsDir<'fs>,
    index: u64
}
impl<'fs, 'd> Iterator for DirNameIter<'fs, 'd> {
    type Item = TskFsName;
    
    fn next(&mut self) -> Option<TskFsName> {
        let tsk_fs_dir_ptr = self.tsk_fs_dir.handle.as_ptr();
        while self.index < unsafe {(*tsk_fs_dir_ptr).names_used} {
            // Get the pointer to the TSK_FS_NAME from the names array at the given index
            let ptr = unsafe {(*tsk_fs_dir_ptr).names.offset(self.index as isize)};

            // Create the TskFsName wrapper for TSK_FS_NAME pointer
            let name_attr = TskFsName::from_ptr(ptr)
                .expect("DirNameIter names ptr is null!");
            
            // Update the index for the next fetch
            self.index += 1;

            // return the TskFsName
            return Some(name_attr);
        }
        
        None
    }
}
