use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    style::{Style, Stylize},
    symbols::scrollbar,
    text::Line,
    widgets::{
        Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};

#[derive(Debug)]
pub struct NotificationPane {
    pub notifications: VecDeque<String>,
}
impl NotificationPane {
    pub fn new(notifications: VecDeque<String>) -> Self {
        Self { notifications }
    }
    //     fn build_notification(text: String) -> Paragraph {
    //         Paragraph::new(text).block(Block::new())
    //     }
    pub fn add_notification(&mut self, text: String) {
        self.notifications.push_front(text);
    }
}
impl Widget for &NotificationPane {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate the area for the scrollable list
        let mut state = ScrollbarState::default();
        // Use the items vector as the source of notifications
        let mut items = vec!["bees".to_string(); self.notifications.len()];
        for _ in 0..100 {
            items.push("TEST".to_string());
        }
        let mut y_offset = area.y;
        // Calculate the height of each notification block (border + padding + text + padding + border)
        let block_height = 1 + 0 + 1 + 0 + 1; // border + padding + text + padding + border
        let available_height = area.height as usize;
        let max_visible = available_height / block_height;
        for notification in items.iter().take(max_visible) {
            let block_area = Rect {
                x: area.x,
                y: y_offset,
                width: area.width,
                height: block_height as u16,
            };
            // Draw the border block
            let block = Block::default()
                .border_type(BorderType::Plain)
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(Style::default().gray());
            block.render(block_area, buf);
            // Draw the notification text inside the block, with 3px padding
            let inner_area = block_area.inner(Margin {
                vertical: 1,
                horizontal: 3,
            });
            let paragraph = Paragraph::new(Line::from(notification.as_str()))
                .alignment(Alignment::Left)
                .gray();
            paragraph.render(inner_area, buf);
            y_offset += block_height as u16;
        }
        // Draw the vertical scrollbar on the left
        Scrollbar::new(ScrollbarOrientation::VerticalLeft)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None)
            .render(
                area.inner(Margin {
                    vertical: 0,
                    horizontal: 0,
                }),
                buf,
                &mut state,
            );
        // -----------------------------------------------
        // //     Scrollbar::new(ScrollbarOrientation::VerticalRight)
        //         .begin_symbol(None)
        //         .end_symbol(None)
        //         .track_symbol(None)
        //         .thumb_symbol("▐")
        //         .render(area, buf, &mut state);
    }
}
