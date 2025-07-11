use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread;

fn main() -> anyhow::Result<()> {
    // 1. Setup PTY system
    let pty_system = NativePtySystem::default();

    // 2. Create a new PTY pair (master/slave)
    let pair = pty_system.openpty(PtySize {
        rows: 30,
        cols: 100,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // 3. Spawn a shell inside the PTY slave
    let cmd = if cfg!(target_os = "windows") {
        CommandBuilder::new("powershell.exe") // or "cmd.exe"
    } else {
        CommandBuilder::new("bash")
    };

    let mut child = pair.slave.spawn_command(cmd)?;
    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

    // 4. Pipe stdout from shell to terminal
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let _ = stdout_lock.write_all(&buffer[..n]);
                    let _ = stdout_lock.flush();
                }
                Err(_) => break,
            }
        }
    });

    // 5. Pipe input from user to shell
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut input_buffer = [0u8; 8192];

    loop {
        match stdin_lock.read(&mut input_buffer) {
            Ok(0) => break,
            Ok(n) => {
                let _ = writer.write_all(&input_buffer[..n]);
                let _ = writer.flush();
            }
            Err(_) => break,
        }
    }

    // 6. Wait for child to exit
    let _ = child.wait()?;

    Ok(())
}
