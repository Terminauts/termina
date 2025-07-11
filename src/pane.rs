use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread;

pub struct Pane {
    pub id: u8,
    pub pty: PtyPair,
}

impl Pane {
    pub fn new(id: u8) -> anyhow::Result<Self> {
        let pty_system = NativePtySystem::default();
        let pty = pty_system.openpty(PtySize {
            rows: 30,
            cols: 100,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = if cfg!(target_os = "windows") {
            CommandBuilder::new("powershell.exe")
        } else {
            CommandBuilder::new("bash")
        };

        let _child = pty.slave.spawn_command(cmd)?;
        let mut reader = pty.master.try_clone_reader()?;

        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let output = String::from_utf8_lossy(&buffer[..n]);
                        print!("{}", output); // TODO: buffer to per-pane memory
                    }
                    _ => break,
                }
            }
        });

        Ok(Self { id, pty })
    }
}
