use std::path::Path;
use std::ptr::NonNull;
use std::ffi::{CStr, CString, c_void};
use crate::{
    errors::TskError,
    bindings as tsk,
    tsk_fs::TskFs,
    tsk_vs::TskVs
};


type ReadCallback = Option<
    unsafe extern "C" fn(
        img: *mut tsk::TSK_IMG_INFO,
        off: tsk::TSK_OFF_T,
        buf: *mut ::std::os::raw::c_char,
        len: usize,
    ) -> isize,
>;
type CloseCallback = Option<unsafe extern "C" fn(arg1: *mut tsk::TSK_IMG_INFO)>;
type ImgStatCallback = Option<unsafe extern "C" fn(arg1: *mut tsk::TSK_IMG_INFO, arg2: *mut tsk::FILE)>;


/// Wrapper for TSK_IMG_INFO
#[derive(Debug)]
pub struct TskImg {
    /// The ptr to the TSK_IMG_INFO struct
    pub handle: NonNull<tsk::TSK_IMG_INFO>
}
impl TskImg {
    /// Create TskImg from custom callbacks
    pub fn from_external(
        ext_img_info: *mut c_void,
        size: i64,
        sector_size: u32,
        read: ReadCallback,
        close: CloseCallback,
        imgstat: ImgStatCallback
    ) -> Result<Self, TskError> {
        // Open TSK_IMG_INFO based off of callback functions
        let tsk_img_info_ptr: *mut tsk::TSK_IMG_INFO = unsafe { 
            tsk::tsk_img_open_external(
                ext_img_info,
                size,
                sector_size,
                read,
                close,
                imgstat
            )
        };

        let handle = match NonNull::new(tsk_img_info_ptr) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                    .ok_or(
                        TskError::lib_tsk_error(
                            format!(
                                "tsk_img_open_external() error. (no context)"
                            )
                        )
                    )?;

                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();

                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!("There was an error opening the img handle: {}", error_msg)
                ));
            },
            Some(h) => h
        };

        Ok( Self { handle })
    }

    /// Create a TskImg wrapper from a given TSK_IMG_INFO NonNull pinter.
    /// 
    pub fn from_tsk_img_info_ptr(img_info: NonNull<tsk::TSK_IMG_INFO>) -> Self {
        Self { handle: img_info }
    }

    /// Create a TskImg wrapper from a given source.
    /// 
    pub fn from_utf8_sing(path: impl AsRef<Path>) -> Result<Self, TskError> {
        // Create a CString for the provided source
        let source = CString::new(path.as_ref().to_string_lossy().as_bytes())
            .map_err(|e| TskError::generic(format!("Unable to create CString from source: {:?}", e)))?;

        // Get a pointer to the TSK_IMG_INFO sturct
        let tsk_img = unsafe {tsk::tsk_img_open_utf8_sing(
            source.as_ptr() as _,
            tsk::TSK_IMG_TYPE_ENUM_TSK_IMG_TYPE_RAW_SING,
            0
        )};
        
        // Ensure that the ptr is not null
        let handle = match NonNull::new(tsk_img) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                    .ok_or(
                        TskError::lib_tsk_error(
                            format!(
                                "There was an error opening the img handle from {}. (no context)",
                                path.as_ref()
                                    .to_string_lossy()
                            )
                        )
                    )?;

                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!("There was an error opening the img handle: {}", error_msg)
                ));
            },
            Some(h) => h
        };

        Ok( Self { handle } )
    }

    /// Get a TskVs at a given offset
    pub fn get_vs_from_offset(&self, offset: u64) -> Result<TskVs, TskError> {
        TskVs::new(&self, offset)
    }

    /// Get a TskFs at a given offset
    pub fn get_fs_from_offset(&self, offset: u64) -> Result<TskFs, TskError> {
        TskFs::from_fs_offset(&self, offset)
    }
}
impl Drop for TskImg {
    fn drop(&mut self) {
        unsafe { tsk::tsk_img_close(self.handle.as_ptr()) };
    }
}