#![allow(non_camel_case_types)]
use std::convert::TryInto;
use std::ffi::CStr;
use std::ptr::NonNull;
use std::io::{Read, Seek, SeekFrom};
use crate::tsk_vs_part::TskVsPart;
use crate::bindings as tsk;

pub struct TskVsPartHandle<'vs, 'p>{
    /// The TskVs that is being used
    tsk_vs_part: &'p TskVsPart<'vs>,
    /// The read pointer
    _offset: i64
}
impl<'vs, 'p> TskVsPartHandle<'vs, 'p> {
    /// Create TskVsPartHandle from TskVsPart
    pub fn new(
        tsk_vs_part: &'p TskVsPart<'vs>
    ) -> Self {
        Self {
            tsk_vs_part,
            _offset: 0
        }
    }
}
impl<'vs, 'p> Seek for TskVsPartHandle<'vs, 'p> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64>{
        match pos {
            SeekFrom::Start(o) => {
                if o > self.tsk_vs_part.size() as u64 {
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Start({}) is greater than partition size {}", o, self.tsk_vs_part.size()
                        )
                    ));
                }
                else {
                    self._offset = o as i64;
                    return Ok(o);
                }
            },
            SeekFrom::Current(o) => {
                let new_offset = self._offset + o;
                if  new_offset > self.tsk_vs_part.size().try_into().unwrap() {
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Current({}) is greater than partition size. new_offset = {} + {} = {}, self.tsk_vs_part.size() = {}",
                            o,
                            self._offset,
                            o,
                            new_offset,
                            self.tsk_vs_part.size()
                        )
                    ));
                }
                else if new_offset < 0 {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!(
                                "Cannot seek Current({}) from offset {}",
                                o,
                                self._offset
                            )
                        )
                    );
                }
                else {
                    self._offset = new_offset;
                }
            },
            SeekFrom::End(o) => {
                let new_offset: u64 = self.tsk_vs_part.size() + o as u64;
                if new_offset > self.tsk_vs_part.size() {
                    return Err(
                        std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Offset Current({}) is greater than attr size. new_offset = {} + {} = {}, self.tsk_vs_part.size() = {}",
                            o,
                            self._offset,
                            o,
                            new_offset,
                            self.tsk_vs_part.size()
                        )
                    ));
                } else {
                    self._offset = new_offset
                        .try_into()
                        .expect("Error converting offset into i64.");
                }
            }
        }

        Ok(self._offset as u64)
    }
}
impl<'vs, 'p> Read for TskVsPartHandle<'vs, 'p> {
    fn read(&mut self, buf: &mut [u8]) ->  std::io::Result<usize> {
        // byte size has to be in i64 due to _offset requried by tsk_vs_part_read
        let part_byte_size = self.tsk_vs_part.size()
            .try_into()
            .expect("Partition size cannot be converted into i64!");

        // Check if offset is at end of partition
        if self._offset == part_byte_size {
            return Ok(0);
        }

        // Read bytes
        let bytes_read = unsafe{tsk::tsk_vs_part_read(
            self.tsk_vs_part.into(),
            self._offset,
            buf.as_mut_ptr() as *mut i8,
            buf.len()
        )};

        match bytes_read {
            -1 => {
                // Get a ptr to the error msg
                let error_msg_ptr = unsafe {NonNull::new(tsk::tsk_error_get() as _)}
                    .ok_or(
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!(
                                "tsk_vs_part_read Error. No context." 
                            )
                        )
                    )?;

                // Get the error message from the string
                let error_msg = unsafe { CStr::from_ptr(error_msg_ptr.as_ptr()) }.to_string_lossy();
                return Err(
                    std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "tsk_vs_part_read Error : {}", error_msg 
                    )
                ));
            }
            _ => {
                    self._offset += TryInto::<i64>::try_into(bytes_read)
                        .unwrap();
                    return Ok(bytes_read as usize);
                }
        };
    }
}
