use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Read};
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::ffi::CStr;
use crate::tsk_img::TskImg;
use crate::errors::TskError;
use crate::bindings as tsk;


pub trait ReadSeek: Read + Seek {
    fn tell(&mut self) -> std::io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
    fn stream_len(&mut self) -> std::io::Result<u64> {
        let old_pos = self.tell()?;
        let len = self.seek(SeekFrom::End(0))?;

        // Avoid seeking a third time when we were already at the end of the
        // stream. The branch is usually way cheaper than a seek operation.
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }

        Ok(len)
    }
}
impl<T: Read + Seek> ReadSeek for T {}


/// Read function for TskImgReadSeek
unsafe extern "C" fn img_read(
    img: *mut tsk::TSK_IMG_INFO,
    off: tsk::TSK_OFF_T,
    buf: *mut ::std::os::raw::c_char,
    len: usize,
) -> isize {
    if img.is_null() {
        error!("img_read contained null TSK_IMG_INFO pointer!");
        return -1;
    }

    // Get pointer to TskImgReadSeekInner
    let reader = img as *mut TskImgReadSeekInner;
    
    // Seek inner stream to offset
    match (*reader).stream.seek(SeekFrom::Start(off as u64)) {
        Ok(pos) => {
            if pos != off as u64 {
                error!("pos {} is not equal to off {}", pos, off);
                return 0;
            }
        },
        Err(error) => {
            eprintln!("Error seeking stream: {:?}", error);
            return 0;
        }
    }

    // Make sure that the buffer's pointer is not null
    if buf.is_null() {
        error!("img_read contained null buffer!");
        return -1;
    }

    // Get slice from buf pointer
    let buffer = core::slice::from_raw_parts_mut(
        buf as *mut u8,
        len as usize
    );
    // Read bytes from stream into buffer
    let bytes_read = match (*reader).stream.read(buffer) {
        Ok(b) => b,
        Err(e) => {
            error!("{:?}", e);
            return 0;
        }
    };

    std::mem::forget(buffer);

    // Check if bytes read was not equal to length of data to read
    if bytes_read != len {
        error!("bytes_read {} is not equal to len {}", bytes_read, len);
        return -1;
    }

    // Return bytes read
    bytes_read.try_into()
        .expect("Cannot convert bytes read into usize.")
}


unsafe extern "C" fn img_info(
    arg1: *mut tsk::TSK_IMG_INFO,
    arg2: *mut tsk::FILE
) {
    trace!("img_info() not implemented");
}


unsafe extern "C" fn img_close(
    arg1: *mut tsk::TSK_IMG_INFO
) {
    trace!("img_close() not implemented");
}


/// Custom struct that will take a ReadSeek trait to read from
#[repr(C)]
struct TskImgReadSeekInner {
    tsk_img_info: tsk::TSK_IMG_INFO,
    stream: Box<dyn ReadSeek>
}
/// TskImgReadSeek uses a boxed read/seek trait and can be turned into a TskImg
pub struct TskImgReadSeek{
    source: String,
    inner: *mut MaybeUninit<TskImgReadSeekInner>,
    tsk_img_info: NonNull<tsk::TSK_IMG_INFO>
}
impl TskImgReadSeek {
    pub fn from_read_seek<S: Into<String>>(
        source: S,
        stream: Box<dyn ReadSeek>,
        size: i64
    ) -> Result<Self, TskError> {
        let source = source.into();
        let sector_size: std::os::raw::c_uint = 512;

        // Create uninitialized reader
        let mut boxed_tsk_reader = Box::<TskImgReadSeekInner>::new_uninit();
        // Set the stream
        unsafe { std::ptr::addr_of_mut!((*boxed_tsk_reader.as_mut_ptr()).stream).write(Box::new(stream)) };
        // Get the pointer to the uninitialized struct
        let tsk_reader_ptr_unint: *mut MaybeUninit<TskImgReadSeekInner> = Box::into_raw(boxed_tsk_reader).cast();

        // Create callback pointers
        let read = Some(img_read as unsafe extern "C" fn(
            img: *mut tsk::TSK_IMG_INFO,
            off: tsk::TSK_OFF_T,
            buf: *mut ::std::os::raw::c_char,
            len: usize,
        ) -> isize);
        let close = Some(
            img_close as unsafe extern "C" fn(
                arg1: *mut tsk::TSK_IMG_INFO
            )
        );
        let imgstat = Some(
            img_info as unsafe extern "C" fn(
                arg1: *mut tsk::TSK_IMG_INFO,
                arg2: *mut tsk::FILE
            )
        );

        // Get address of the TSK_IMG_INFO pointer
        let img_addr = unsafe{ std::ptr::addr_of_mut!(
            (*(*tsk_reader_ptr_unint).as_mut_ptr()).tsk_img_info
        )};
        // Open via the external source
        let tsk_img_info_ptr: *mut tsk::TSK_IMG_INFO = unsafe { 
            tsk::tsk_img_open_external(
                img_addr as _,
                size,
                sector_size,
                read,
                close,
                imgstat
            )
        };

        // Get non null pointer
        let tsk_img_info_ptr = match NonNull::new(tsk_img_info_ptr) {
            None => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe { NonNull::new(tsk::tsk_error_get() as _) }
                    .ok_or(
                        TskError::lib_tsk_error(
                            format!(
                                "There was an error opening read/seek img handle from {}. (no context)",
                                &source
                            )
                        )
                    )?;

                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
                // Return an error which includes the TSK error message
                return Err(TskError::lib_tsk_error(
                    format!(
                        "There was an error opening read/seek img handle from {}: {}",
                        &source,
                        error_msg
                    )
                ));
            },
            Some(h) => h
        };

        Ok( Self{
            source,
            inner: tsk_reader_ptr_unint,
            tsk_img_info: tsk_img_info_ptr
        })
    }
}
impl std::fmt::Debug for TskImgReadSeek {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("TskImgReadSeek")
         .field("source", &self.source)
         .field("inner", &self.inner)
         .field("tsk_img_info", &self.tsk_img_info)
         .finish()
    }
}
impl<'fs> Into<TskImg> for TskImgReadSeek {
    fn into(self) -> TskImg {
        TskImg::from_tsk_img_info_ptr(self.tsk_img_info)
    }
}
