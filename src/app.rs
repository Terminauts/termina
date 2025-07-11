use crate::pane::Pane;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
};
use std::collections::HashMap;

pub struct App {
    pub panes: HashMap<u8, Pane>,
    pub focused: u8,
    pub next_id: u8,
    pub command_buffer: String,
}

impl App {
    pub fn new() -> Self {
        let mut panes = HashMap::new();
        panes.insert(0, Pane::new(0).expect("Failed to create pane"));
        Self {
            panes,
            focused: 0,
            next_id: 1,
            command_buffer: String::new(),
        }
    }

    pub fn split_vertical(&mut self) {
        let id = self.next_id;
        if let Ok(pane) = Pane::new(id) {
            self.panes.insert(id, pane);
            self.focused = id;
            self.next_id += 1;
        }
    }

    pub fn switch_focus(&mut self) {
        let mut keys: Vec<u8> = self.panes.keys().copied().collect();
        keys.sort();
        if let Some(pos) = keys.iter().position(|&x| x == self.focused) {
            let next = (pos + 1) % keys.len();
            self.focused = keys[next];
        }
    }

    pub fn draw(&self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                (0..self.panes.len())
                    .map(|_| Constraint::Percentage(100 / self.panes.len() as u16))
                    .collect::<Vec<_>>(),
            )
            .split(f.size());

        for ((&id, pane), area) in self.panes.iter().zip(chunks.iter()) {
            let block = Block::default()
                .title(format!("Pane {}", id))
                .borders(Borders::ALL)
                .style(if id == self.focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

            let mut output = pane.get_output();
            // if id == self.focused {
            //     output.push_str(&self.command_buffer);
            // }
            let paragraph = Paragraph::new(output)
                .block(block)
                .wrap(Wrap { trim: false });

            f.render_widget(paragraph, *area);
        }
    }
}
