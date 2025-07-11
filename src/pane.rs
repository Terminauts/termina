use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
use std::{
    io::Read,
    sync::{Arc, Mutex},
    thread,
};
#[allow(dead_code)]
pub struct Pane {
    pub id: u8,
    pub pty: PtyPair,
    pub writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    pub buffer: Arc<Mutex<Vec<String>>>,
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

        let cmd = if cfg!(target_os = "windows") {
            CommandBuilder::new("powershell.exe")
        } else {
            CommandBuilder::new("bash")
        };

        let _child = pty.slave.spawn_command(cmd)?;
        let mut reader = pty.master.try_clone_reader()?;
        let writer = pty.master.take_writer().unwrap();

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = Arc::clone(&buffer);

        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let text = String::from_utf8_lossy(&buf[..n]).to_string();
                let mut locked = buffer_clone.lock().unwrap();
                locked.push(text);
                if locked.len() > 100 {
                    locked.remove(0);
                }
            }
        });

        Ok(Self {
            id,
            pty,
            writer: Arc::new(Mutex::new(writer)),
            buffer,
        })
    }

    pub fn send_input(&self, input: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(input);
            let _ = w.flush();
        }
        // if let Ok(mut buffer) = self.buffer.lock() {
        //     buffer.clear(); // fagia output
        // }
    }

    pub fn get_output(&self) -> String {
        let locked = self.buffer.lock().unwrap();
        locked.join("")
    }
}
