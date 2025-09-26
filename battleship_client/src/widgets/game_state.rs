use std::error::Error;

use battleship_models::{CellStates, GameStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph, Widget},
};
use tracing::info;

#[derive(Debug)]
pub struct GameState {
    pub player_name: String,
    pub rows: u16,
    pub cols: u16,
    pub board: Vec<Vec<CellStates>>,
    pub state: GameStatus,
    pub position: Position,
}
impl GameState {
    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let Position { x, y } = self.position;
        self.position.x = (x as i16 + dx).clamp(0, self.cols as i16 - 1) as u16;
        self.position.y = (y as i16 + dy).clamp(0, self.rows as i16 - 1) as u16;
        info!(
            "Resulting position of ({}, {})",
            self.position.x, self.position.y
        );
    }
    pub fn destroy_ship(&mut self) -> Result<(), Box<dyn Error>> {
        self.update_cell(self.position.y, self.position.x, CellStates::Hit)
    }
    pub fn place_ship(&mut self) -> Result<(), Box<dyn Error>> {
        self.update_cell(self.position.y, self.position.x, CellStates::Boat)
    }
    pub fn update_cell(
        &mut self,
        row: u16,
        col: u16,
        state: CellStates,
    ) -> Result<(), Box<dyn Error>> {
        if row > 0 && row < self.rows && col > 0 && col < self.cols {
            self.board[row as usize][col as usize] = state;
            Ok(())
        } else {
            Err("".into())
        }
    }
}
impl Widget for &GameState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let get_block = || {
            Block::default()
                .border_type(ratatui::widgets::BorderType::Plain)
                .borders(ratatui::widgets::Borders::ALL)
        };
        let col_constraints = (0..self.cols).map(|_| Constraint::Ratio(1, self.cols as u32));
        let row_constraints = (0..self.rows).map(|_| Constraint::Ratio(1, self.rows as u32));
        let horizontal = Layout::horizontal(col_constraints).spacing(1);
        let vertical = Layout::vertical(row_constraints).spacing(1);

        let rows = vertical.split(area);
        let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

        for (i, cell) in cells.enumerate() {
            let row = i as u16 / self.cols;
            let col = i as u16 % self.cols;
            let b = if row == self.position.y && col == self.position.x {
                get_block().border_style(Style::default().fg(Color::Yellow))
            } else {
                get_block().border_style(Style::default().fg(Color::Blue))
            };
            let c = Paragraph::new(format!("Area {:02}", i + 1)).block(b);
            c.render(cell, buf);
        }
    }
}
