mod commands;

use crate::errors::Error;
use crate::errors::Error::{CRCVerificationError, PipeError};
use crate::frame::frame_flags::Flag::{CodecJSON, Control};
use crate::frame::{Frame, WORD};
use crate::pipe::commands::PidCommand;
use std::process::Stdio;
use std::str::from_utf8;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};

pub struct Pipes {
    pub child: Child,
}

pub trait Marshaller {
    fn marshal(&mut self) -> Result<Vec<u8>, Error>;
}

impl Pipes {
    pub async fn send(&mut self, frame: &mut Frame) -> Result<(), Error> {
        let stdin = self.child.stdin.as_mut();

        match stdin {
            None => Err(PipeError {
                cause: "no stdin".to_string(),
            }),
            Some(child) => {
                child.write_all(&frame.bytes()).await?;
                Ok(())
            }
        }
    }

    pub async fn receive_stderr(&mut self) -> Result<Vec<u8>, Error> {
        let stderr = self.child.stderr.as_mut();
        match stderr {
            // no data
            None => Ok(vec![]),
            // some data
            Some(child) => {
                let mut data = vec![];
                child.read_to_end(&mut data).await?;
                Ok(data)
            }
        }
    }

    pub async fn receive_stdout(&mut self) -> Result<Frame, Error> {
        let stdout = self.child.stdout.as_mut();
        match stdout {
            None => Err(PipeError {
                cause: String::from("nothing in the stdout"),
            }),

            Some(child) => {
                let mut buf = BufReader::new(child);
                let mut fr = Frame::default();

                // read only header, 12 bytes
                buf.read_exact(fr.header_mut()).await?;

                // we have an options
                if fr.read_hl() > 3 {
                    let opts_len = (fr.read_hl() - 3) * WORD;
                    let mut tmp = vec![0; opts_len as usize];
                    buf.read_exact(&mut tmp).await?;

                    fr.extend_header(&tmp);
                }

                if fr.verify_crc().is_err() {
                    let mut buffer = vec![];
                    let timeout_dur = Duration::from_secs(2);
                    _ = timeout(timeout_dur, buf.read_to_end(&mut buffer)).await;

                    // TODO: handle error match here!
                    let msg = match from_utf8(fr.header()) {
                        Ok(m) => String::from(m),
                        Err(_) => String::new(),
                    };

                    let bufmsg = match from_utf8(&buffer) {
                        Ok(m) => String::from(m),
                        Err(_) => String::new(),
                    };

                    return Err(CRCVerificationError {
                        cause: format!("{}{}", msg, bufmsg),
                    });
                }

                let pld_len = fr.read_payload_len();
                if pld_len == 0 {
                    return Ok(fr);
                }

                buf.read_exact(fr.init_payload_mut(pld_len as usize))
                    .await?;

                Ok(fr)
            }
        }
    }

    pub async fn send_control<T: Marshaller>(&mut self, mut payload: T) -> Result<(), Error> {
        let mut frame = Frame::default();

        frame.write_version(1);
        frame.write_flags(&[Control, CodecJSON]);

        let data = payload.marshal()?;

        frame.write_payload(data);
        frame.write_crc();

        if let Some(socket_new) = self.child.stdin.as_mut() {
            let data = frame.bytes();
            socket_new.write_all(&data).await?;
            return Ok(());
        }

        Err(PipeError {
            cause: String::from("get None child stdin out from the option"),
        })
    }

    pub async fn pid(&mut self) -> Result<u32, Error> {
        self.send_control(PidCommand::default()).await?;

        let f = self.receive_stdout().await?;

        let flags = f.read_flags();
        if flags & (Control as u8) == 0 {
            return Err(PipeError {
                cause: String::from("unexpected response, header is missing, no CONTROL flag"),
            });
        }

        let payload = f.payload();
        let res: PidCommand = serde_json::from_slice(payload).unwrap();

        if res.pid == 0 {
            return Err(PipeError {
                cause: String::from("pid should be greater than 0"),
            });
        }

        Ok(res.pid)
    }

    pub async fn kill(&mut self) -> Result<(), Error> {
        self.child.kill().await?;
        Ok(())
    }
}

impl Pipes {
    pub fn new(cmd: &[&str]) -> Result<Self, Error> {
        // TODO check the input
        let command = Command::new(cmd[0])
            .args(&cmd[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Pipes { child: command })
    }
}

mod tests {
    #[tokio::test]
    async fn test1() {
        use crate::frame::Frame;
        use crate::pipe::Pipes;

        let mut p = Pipes::new(&["php", "tests/worker.php"]).unwrap();
        let mut frame = Frame::default();
        let payload = vec![b'h', b'e', b'l', b'l', b'o'];

        frame.write_version(1);
        frame.write_flags(&[]);
        frame.write_options(&[0]);
        frame.write_payload(payload);
        frame.write_crc();

        p.send(&mut frame).await.unwrap();

        match p.receive_stdout().await {
            Ok(mut data) => {
                println!("{:?}", data.bytes());
            }
            Err(error) => {
                assert_eq!(error.to_string(), "validation failed on the message sent to STDOUT, cause warning: some weird php error, THIS IS PHP, I'm THE KING :) \u{14}\0\u{5}\0\0\0\u{1c}\u{11}[\u{1e}\0\0\0\0\0\0hello");
                println!("{:?}", error.to_string());
            }
        }
    }

    #[tokio::test]
    async fn test2() {
        use crate::pipe::Pipes;

        let mut p = Pipes::new(&["php", "tests/worker.php"]).unwrap();
        if let Ok(pid) = p.pid().await {
            assert!(pid > 0);
        }
    }
}
