mod app;
mod pane;
mod terminal;

use app::App;
use terminal::{restore_terminal, setup_terminal};

use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Non-blocking input check
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        restore_terminal()?;
                        break;
                    }
                    KeyCode::Char('v') => app.split_vertical(),
                    KeyCode::Tab => app.switch_focus(),
                    KeyCode::Char(c) => app.command_buffer.push(c),
                    KeyCode::Backspace => {
                        app.command_buffer.pop();
                    }
                    KeyCode::Enter => {
                        if let Some(pane) = app.panes.get(&app.focused) {
                            let line = app.command_buffer.clone() + "\n";
                            pane.send_input(line.as_bytes());
                            app.command_buffer.clear();
                        }
                    }
                    _ => {}
                }
            }
        }

        // ✅ Always redraw after input or timeout
        if last_tick.elapsed() >= tick_rate {
            terminal.draw(|f| app.draw(f))?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}
