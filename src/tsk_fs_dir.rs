use std::ffi::CStr;
use std::ptr::NonNull;
use crate::{
    errors::TskError,
    tsk_fs::TskFs,
    tsk_fs_name::TskFsName,
    bindings as tsk
};


/// Wrapper for TSK_FS_DIR that implements helper functions.
#[derive(Clone)]
pub struct TskFsDir<'fs> {
    /// A TskFsDir can never outlive its TskFs
    tsk_fs_info_ptr: &'fs *mut tsk::TSK_FS_INFO,
    /// The ptr to the TSK_FS_DIR struct
    tsk_fs_dir_ptr: *mut tsk::TSK_FS_DIR,
    _release: bool
}
impl<'fs> TskFsDir<'fs> {
    /// Create a TSK_FS_DIR wrapper given TskFs and inode
    pub fn from_meta(tsk_fs: &'fs TskFs, inode: u64) -> Result<Self, TskError> {
        // Get a pointer to the TSK_FS_DIR sturct
        let tsk_fs_dir_ptr = unsafe {tsk::tsk_fs_dir_open_meta(
            tsk_fs.into(),
            inode as _
        )};

        // Ensure that the ptr is not null
        if tsk_fs_dir_ptr.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                .ok_or(TskError::lib_tsk_error(
                    format!("There was an error opening {} as a dir. No context.", inode)
                ))?;

            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::lib_tsk_error(
                format!("There was an error opening {} as a dir: {}", inode, error_msg)
            ));
        }

        Ok( Self { 
            tsk_fs_info_ptr: tsk_fs.into(), 
            tsk_fs_dir_ptr,
            _release: true
        } )
    }

    /// Get a TskFsName at a given index of the TSK_FS_DIR
    pub fn get_name(&self, index: u64) -> Result<TskFsName, TskError> {
        // Get a pointer to the TSK_FS_FILE sturct
        let tsk_fs_name = unsafe {tsk::tsk_fs_dir_get_name(
            self.tsk_fs_dir_ptr,
            index as _
        )};

        if tsk_fs_name.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                .ok_or(TskError::tsk_fs_name_error(
                    format!("Error getting TskFsName at index {} from TskFsDir {:?}. No context.", index, &self)
                ))?;
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
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

    /// Get an iterator that iterates TskFsNames of this TskFsDir
    pub fn into_name_iter(self) -> IntoDirNameIter<'fs> {
        IntoDirNameIter {
            tsk_fs_dir: self,
            index: 0
        }
    }

    /// Get the mut ptr for TSK_FS_DIR 
    pub fn as_mut_ptr(&mut self) -> *mut tsk::TSK_FS_DIR {
        self.tsk_fs_dir_ptr
    }
}
impl<'fs> Into<*mut tsk::TSK_FS_DIR> for &TskFsDir<'fs> {
    fn into(self) -> *mut tsk::TSK_FS_DIR {
        self.tsk_fs_dir_ptr
    }
}
impl<'fs> Into<*mut tsk::TSK_FS_DIR> for TskFsDir<'fs> {
    fn into(self) -> *mut tsk::TSK_FS_DIR {
        self.tsk_fs_dir_ptr
    }
}
impl<'fs, 'd> Into<&'d *mut tsk::TSK_FS_DIR> for &'d mut TskFsDir<'fs> {
    fn into(self) -> &'d *mut tsk::TSK_FS_DIR {
        &self.tsk_fs_dir_ptr
    }
}
impl<'fs, 'd> Into<&'d *mut tsk::TSK_FS_DIR> for &'d TskFsDir<'fs> {
    fn into(self) -> &'d *mut tsk::TSK_FS_DIR {
        &self.tsk_fs_dir_ptr
    }
}
impl<'fs> std::fmt::Debug for TskFsDir<'fs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TskFsDir")
         .field("addr", unsafe{&(*self.tsk_fs_dir_ptr).addr})
         .field("seq", unsafe{&(*self.tsk_fs_dir_ptr).seq})
         .field("fs_file", unsafe{&(*self.tsk_fs_dir_ptr).fs_file})
         .field("names_used", unsafe{&(*self.tsk_fs_dir_ptr).names_used})
         .field("names_alloc", unsafe{&(*self.tsk_fs_dir_ptr).names_alloc})
         .field("names", unsafe{&(*self.tsk_fs_dir_ptr).names})
         .finish()
    }
}
impl<'fs> Drop for TskFsDir<'fs> {
    fn drop(&mut self) {
        if self._release {
            unsafe { tsk::tsk_fs_dir_close(self.tsk_fs_dir_ptr) };
        }
    }
}


/// DirNameIter is an iterator for the allocated TskFsName of a given TskFsDir.
#[derive(Debug, Clone)]
pub struct IntoDirNameIter<'fs>{
    tsk_fs_dir: TskFsDir<'fs>,
    index: usize
}
impl<'fs> Iterator for IntoDirNameIter<'fs> {
    type Item = TskFsName;
    
    fn next(&mut self) -> Option<TskFsName> {
        let tsk_fs_dir_ptr: *mut tsk::TSK_FS_DIR = self.tsk_fs_dir.as_mut_ptr();
        let names_used =  unsafe {
            (*tsk_fs_dir_ptr).names_used
        };

        if self.index < names_used {
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


/// DirNameIter is an iterator for the allocated TskFsName of a given TskFsDir.
#[derive(Debug, Clone)]
pub struct DirNameIter<'fs, 'd>{
    tsk_fs_dir: &'d TskFsDir<'fs>,
    index: usize
}
impl<'fs, 'd> DirNameIter<'fs, 'd> {
    pub fn get_dir(&self) -> &'d TskFsDir<'fs> {
        self.tsk_fs_dir
    }
}
impl<'fs, 'd> Iterator for DirNameIter<'fs, 'd> {
    type Item = TskFsName;
    
    fn next(&mut self) -> Option<TskFsName> {
        let tsk_fs_dir_ptr: *mut tsk::TSK_FS_DIR = self.tsk_fs_dir.into();

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
