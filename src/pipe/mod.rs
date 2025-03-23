mod commands;

use crate::frame::frame_flags::Flag::{CodecJSON, Control};
use crate::frame::{Frame, WORD};
use crate::pipe::commands::PidCommand;
use anyhow::anyhow;
use std::process::Stdio;
use std::str::from_utf8;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{Duration, timeout};

pub struct Pipes {
    pub child: Child,
}

pub trait Marshaller {
    fn marshal(&mut self) -> anyhow::Result<Vec<u8>>;
}

impl Pipes {
    pub async fn send(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let stdin = self.child.stdin.as_mut();

        match stdin {
            None => Err(anyhow!("no stdin")),
            Some(child) => {
                child.write_all(&frame.bytes()).await?;
                Ok(())
            }
        }
    }

    pub async fn receive_stderr(&mut self) -> anyhow::Result<Vec<u8>> {
        let stderr = self.child.stderr.as_mut();
        match stderr {
            // no data
            None => Err(anyhow!("no data, process is possibly dead")),
            // some data
            Some(child) => {
                let mut data = vec![];
                child.read_to_end(&mut data).await?;
                Ok(data)
            }
        }
    }

    pub async fn receive_stdout(&mut self) -> anyhow::Result<Frame> {
        let stdout = self.child.stdout.as_mut();
        match stdout {
            None => Err(anyhow!("no data, process is possibly dead")),

            Some(child) => {
                let mut buf = BufReader::new(child);
                let mut fr = Frame::default();

                // read-only header, 12 bytes
                buf.read_exact(fr.header_mut()).await?;

                // we have an option
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

                    return Err(anyhow!(
                        "validation failed on the message sent to STDOUT, cause {}{}",
                        msg,
                        bufmsg
                    ));
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

    pub async fn send_control<T: Marshaller>(&mut self, mut payload: T) -> anyhow::Result<()> {
        let mut frame = Frame::default();

        frame.write_version(1);
        frame.write_flags(&[Control, CodecJSON]);

        let data = payload.marshal()?;
        // we don't need to Borrow the data here
        frame.write_payload(&data);
        frame.write_crc();

        if let Some(socket_new) = self.child.stdin.as_mut() {
            let data = frame.bytes();
            socket_new.write_all(&data).await?;
            return Ok(());
        }

        Err(anyhow!("get None child stdin out from the option"))
    }

    pub async fn pid(&mut self) -> anyhow::Result<u32> {
        self.send_control(PidCommand::default()).await?;

        let f = self.receive_stdout().await?;

        let flags = f.read_flags();
        if flags & (Control as u8) == 0 {
            return Err(anyhow!(
                "unexpected response, header is missing, no CONTROL flag"
            ));
        }

        let payload = f.payload();
        let res: PidCommand = serde_json::from_slice(payload)?;

        if res.pid == 0 {
            return Err(anyhow!("pid should be greater than 0"));
        }

        Ok(res.pid)
    }

    pub async fn kill(&mut self) -> anyhow::Result<()> {
        self.child.kill().await?;
        Ok(())
    }
}

impl Pipes {
    pub async fn new(cmd: &[&str]) -> anyhow::Result<Self> {
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

        let mut p = Pipes::new(&["php", "tests/worker.php"]).await.unwrap();
        let mut frame = Frame::default();
        let payload = vec![b'h', b'e', b'l', b'l', b'o'];

        frame.write_version(1);
        frame.write_flags(&[]);
        frame.write_options(&[0]);
        frame.write_payload(&payload);
        frame.write_crc();

        p.send(&mut frame).await.unwrap();

        match p.receive_stdout().await {
            Ok(data) => {
                assert_eq!(data.payload(), &payload);
            }
            Err(error) => {
                assert_eq!(
                    error.to_string(),
                    "validation failed on the message sent to STDOUT, cause warning: some weird php error, THIS IS PHP, I'm THE KING :) \u{14}\0\u{5}\0\0\0\u{1c}\u{11}[\u{1e}\0\0\0\0\0\0hello"
                );
                println!("{:?}", error.to_string());
            }
        }
    }
}
