use std::ffi::CStr;
use crate::{
    errors::TskError,
    tsk_img::TskImg,
    tsk_fs_file::TskFsFile,
    tsk_fs_name::TskFsName,
    tsk_fs_dir::{TskFsDir, IntoDirNameIter},
    bindings as tsk
};


/// Wrapper for TSK_FS_INFO 
#[derive(Debug)]
pub struct TskFs {
    /// The ptr to the TSK_FS_INFO struct
    tsk_fs_ptr: *mut tsk::TSK_FS_INFO,
    /// We dont always want to free a file pointer
    _release: bool,
}
impl TskFs {
    /// Create a TSK_FS_INFO wrapper given the TskImg and offset of the file system
    pub fn from_fs_offset(tsk_img: &TskImg, offset: u64) -> Result<TskFs, TskError> {
        // Get a pointer to the TSK_FS_INFO sturct
        let tsk_fs_ptr = unsafe {tsk::tsk_fs_open_img(
            tsk_img.handle.as_ptr(),
            offset as i64 as _,
            0
        )};

        if tsk_fs_ptr.is_null() {
            // Get a ptr to the error msg
            let error_msg_ptr = unsafe { tsk::tsk_error_get() };
            // Get the error message from the string
            let error_msg = unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy();
            // Return an error which includes the TSK error message
            return Err(TskError::lib_tsk_error(
                format!("There was an error opening the fs handle at offset {}: {}", offset, error_msg)
            ));
        }

        Ok( Self { tsk_fs_ptr, _release: true } )
    }

    /// Open a file by a given path. (use '/' as separators)
    pub fn file_open(&self, path: &str) -> Result<TskFsFile, TskError> {
        TskFsFile::from_path(&self, path)
    }

    /// Open a file by a given inode.
    pub fn file_open_meta(&self, inode: u64) -> Result<TskFsFile, TskError> {
        TskFsFile::from_meta(&self, inode)
    }

    pub fn iter_file_names<'fs>(&'fs self) -> Result<FsNameIter<'fs>, TskError> {
        let root_inode = unsafe {(*self.tsk_fs_ptr).root_inum};
        let root_dir = TskFsDir::from_meta(self, root_inode)?;

        Ok( FsNameIter {
            tsk_fs: self,
            dir_iter_stack: vec![root_dir.into_name_iter()],
            path_stack: Vec::new()
        } )
    }
}
impl Into<*mut tsk::TSK_FS_INFO> for &TskFs {
    fn into(self) -> *mut tsk::TSK_FS_INFO {
        self.tsk_fs_ptr
    }
}
impl Into<*mut tsk::TSK_FS_INFO> for TskFs {
    fn into(self) -> *mut tsk::TSK_FS_INFO {
        self.tsk_fs_ptr
    }
}
impl<'fs> Into<&'fs *mut tsk::TSK_FS_INFO> for &'fs TskFs {
    fn into(self) -> &'fs *mut tsk::TSK_FS_INFO {
        &self.tsk_fs_ptr
    }
}
impl Drop for TskFs {
    fn drop(&mut self) {
        if self._release {
            unsafe { tsk::tsk_fs_close(self.tsk_fs_ptr) };
        }
    }
}


#[derive(Debug)]
pub struct FsNameIter<'fs> {
    tsk_fs: &'fs TskFs,
    dir_iter_stack: Vec<IntoDirNameIter<'fs>>,
    path_stack: Vec<String>
}
impl<'fs> Iterator for FsNameIter<'fs> {
    type Item = (String, TskFsName);

    fn next(&mut self) -> Option<(String, TskFsName)> {
        let fs = self.tsk_fs;

        loop {
            if let Some(current_dir_itr) = self.dir_iter_stack.last_mut() {
                if let Some(tsk_fn) = current_dir_itr.next() {
                    let inode = tsk_fn.get_inode();

                    if tsk_fn.is_dir() {
                        let file_name = match tsk_fn.name() {
                            Some(n) => n,
                            None => break
                        };

                        if &file_name == "." || &file_name == ".." {
                            continue;
                        }

                        let tsk_fs_dir = match TskFsDir::from_meta(fs, inode) {
                            Ok(d) => d,
                            Err(_e) => continue
                        };

                        let new_dir_iter = tsk_fs_dir.into_name_iter();
                        self.dir_iter_stack.push(new_dir_iter);

                        self.path_stack.push(file_name);
                        
                        let path = self.path_stack.join("/");
                        return Some((path, tsk_fn))
                    } else {
                        let path = self.path_stack.join("/");
                        return Some((path, tsk_fn))
                    }
                } else {
                    self.dir_iter_stack.pop();
                    self.path_stack.pop();
                }
            } else {
                self.dir_iter_stack.pop();

                // No more dir iterators in stack
                if self.dir_iter_stack.is_empty(){
                    break;
                }
            }
        }

        None
    }
}