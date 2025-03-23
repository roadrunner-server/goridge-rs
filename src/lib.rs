#![warn(rust_2018_idioms)]

use std::fs::File;
pub mod bit_operations;
pub mod errors;
pub mod frame;
pub mod pipe;
mod unix;

pub struct PipeReader(File);

impl PipeReader {
    //pub fn try_clone(&self) -> io::Result<PipeReader> {
    //  Ok(PipeReader)
    //}
}
