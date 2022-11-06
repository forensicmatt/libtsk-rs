#![allow(non_camel_case_types)]
use crate::tsk_vs_part::TskVsPart;


pub struct TskVsPartHandle<'p>{
    /// The TskVs that is being used
    tsk_vs_part: &'p TskVsPart,
    /// The read pointer
    _offset: i64
}
impl<'p> TskVsPartHandle<'p> {
    /// Create TskVsPartHandle from TskVsPart
    pub fn new(
        tsk_vs_part: &'p TskVsPart
    ) -> Self {
        Self {
            tsk_vs_part,
            _offset: 0
        }
    }
}