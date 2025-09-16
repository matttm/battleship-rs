use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Block, BorderType, Widget}
};

#[derive(Debug)]
pub struct NotificationPane {
    pub notifications: VecDeque<String>,
}

impl Widget for NotificationPane {
        fn render(self, area: Rect, buf: &mut Buffer) {
            let block = Block::bordered()
                .title("{{project-name}}")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);
            block.render(area, buf);
        }
}
