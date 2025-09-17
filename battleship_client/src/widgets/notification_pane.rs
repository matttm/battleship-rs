use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Block, BorderType, Widget},
};

#[derive(Debug)]
pub struct NotificationPane {
    pub notifications: VecDeque<String>,
}
impl NotificationPane {
    pub fn new(notifications: VecDeque<String>) -> Self {
        Self { notifications }
    }
}
impl Widget for &NotificationPane {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("{{project-name}}")
            .title_alignment(Alignment::Right);
        block.render(area, buf);
    }
}
