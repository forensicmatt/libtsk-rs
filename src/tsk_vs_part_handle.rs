#![allow(non_camel_case_types)]
use std::{io::{Read, Seek, SeekFrom}, convert::TryInto};
use crate::tsk_vs_part::TskVsPart;


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
                    self._offset = new_offset
                        .try_into()
                        .expect("Error converting offset into i64.");
                }
            }
        }

        Ok(self._offset as u64)
    }
}