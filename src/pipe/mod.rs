use crate::errors::Error;
use crate::frame::Frame;
use crate::relay::Relay;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{ChildStderr, ChildStdin, ChildStdout, Command, Stdio};

pub struct Pipes {
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}

impl Relay<Frame> for Pipes {
    fn send(&mut self, frame: &mut Frame) -> Result<(), Error> {
        let stdin= self.stdin.as_mut();

        match stdin {
            None => Err(Error::PipeError {
                cause: "no stdin".to_string(),
            }),
            Some(child) => {
                let mut buf = BufWriter::new(child);
                match buf.write_all(&frame.bytes()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(Error::PipeError {
                        cause: err.to_string(),
                    }),
                }
            }
        }
    }

    ///
    ///
    ///
    ///
    ///
    ///
    ///
    fn receive_stderr(&mut self) -> Result<Vec<u8>, Error> {
        let stderr = self.stderr.as_mut();
        match stderr {
            // no data
            None => Ok(vec![]),
            // some data
            Some(child) => {
                let mut buf = BufReader::new(child);
                let mut data = vec![];

                match buf.read_to_end(&mut data) {
                    Ok(_) => Ok(data),
                    Err(err) => Err(Error::PipeError {
                        cause: err.to_string(),
                    }),
                }
            }
        }
    }

    fn receive_stdout(&mut self) -> Result<Frame, Error> {
        let stdout = self.stdout.as_mut();
        match stdout {
            None => Err(Error::PipeError {
                cause: "".to_string(),
            }),
            Some(child) => {
                let mut buf = BufReader::new(child);
                let mut fr = Frame::default();

                // read only header, 12 bytes
                buf.read_exact(&mut fr.header())?;

                let mut data = vec![];
                match buf.read_to_end(&mut data) {
                    Ok(_) => Ok(Frame::from(data)),
                    Err(err) => Err(Error::PipeError {
                        cause: err.to_string(),
                    }),
                }
            }
        }
    }
}

impl Pipes {
    fn new(cmd: &[&str]) -> Result<Self, std::io::Error> {
        let command = Command::new(cmd[0])
            .args(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Pipes {
            stderr: command.stderr,
            stdout: command.stdout,
            stdin: command.stdin,
        })
    }
}

mod tests {
    use crate::pipe::Pipes;
    use crate::relay::Relay;
    use std::process::Stdio;

    #[test]
    fn test1() {
        let cmd: &str = "ls";

        let mut p = Pipes::new(&["php", "/home/valery/projects/opensource/github/rustatian/roadrunner-rs/crates/goridge/tests/worker.php"]).unwrap();
        let data = p.receive_stderr().unwrap();
        println!("{:?}", std::str::from_utf8(&data));
        let data2 = p.receive_stderr().unwrap();
        println!("{:?}", std::str::from_utf8(&data2));
    }
}
