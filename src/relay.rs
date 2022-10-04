use crate::errors::Error;
use crate::frame::Frame;

pub trait Relay<T> {
    fn send(&mut self, frame: &mut T) -> Result<(), Error>;
    fn receive_stderr(&mut self) -> Result<Vec<u8>, Error>;
    fn receive_stdout(&mut self) -> Result<Frame, Error>;
}
