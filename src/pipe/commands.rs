use crate::errors::Error;
use crate::pipe::Marshaller;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PidCommand {
    pub pid: u32,
}

impl Default for PidCommand {
    fn default() -> Self {
        Self {
            pid: std::process::id(),
        }
    }
}

impl Marshaller for PidCommand {
    fn marshal(&mut self) -> Result<Vec<u8>, Error> {
        match serde_json::to_vec(self) {
            Ok(data) => Ok(data),
            Err(error) => Err(Error::PipeError {
                cause: error.to_string(),
            }),
        }
    }
}

#[derive(Serialize, Default)]
struct StopCommand {
    stop: bool,
}

impl Marshaller for StopCommand {
    fn marshal(&mut self) -> Result<Vec<u8>, Error> {
        self.stop = true;
        match serde_json::to_vec(self) {
            Ok(data) => Ok(data),
            Err(error) => Err(Error::PipeError {
                cause: error.to_string(),
            }),
        }
    }
}
