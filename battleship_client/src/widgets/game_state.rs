use std::error::Error;

use battleship_models::{CellState, GameStatus};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Paragraph, Widget},
};
use tracing::info;

#[derive(Debug)]
pub struct GameState {
    pub id: String,
    pub player_name: String,
    pub rows: usize,
    pub cols: usize,
    pub board: Vec<Vec<CellState>>,
    pub status: GameStatus,
    pub position: Position,
}
#[derive(Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
impl GameState {
    pub fn clear_board(&mut self) {
        // Iterate over rows with mutable references
        for row in self.board.iter_mut() {
            // Iterate over elements within each row with mutable references
            for cell in row.iter_mut() {
                *cell = CellState::Empty;
            }
        }
    }
    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let Position { x, y } = self.position;
        self.position.x = (x as i16 + dx).clamp(0, self.cols as i16 - 1) as usize;
        self.position.y = (y as i16 + dy).clamp(0, self.rows as i16 - 1) as usize;
        info!(
            "Resulting position of ({}, {})",
            self.position.x, self.position.y
        );
    }
    pub fn destroy_ship(&mut self, y: usize, x: usize) -> Result<(), Box<dyn Error>> {
        self.update_cell(y, x, CellState::Hit)
    }
    pub fn place_ship(&mut self, y: usize, x: usize) -> Result<(), Box<dyn Error>> {
        self.update_cell(y, x, CellState::Boat)
    }
    pub fn mark_ship_pending(&mut self, y: usize, x: usize) -> Result<(), Box<dyn Error>> {
        self.update_cell(y, x, CellState::Pending)
    }
    pub fn update_cell(
        &mut self,
        row: usize,
        col: usize,
        state: CellState,
    ) -> Result<(), Box<dyn Error>> {
        if row < self.rows && col < self.cols {
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
        let horizontal = Layout::horizontal(col_constraints).spacing(0);
        let vertical = Layout::vertical(row_constraints).spacing(0);
        let rows = vertical.split(area);
        let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());
        for (i, cell) in cells.enumerate() {
            let row = i / self.cols;
            let col = i % self.cols;
            let cell_state = self.board[row][col];
            // color according to state
            let color = match &cell_state {
                CellState::Pending => Color::White,
                CellState::Empty => Color::Black,
                CellState::Boat => Color::Green,
                CellState::Miss => Color::DarkGray,
                CellState::Hit => Color::Red,
            };
            // mark border if positioned on cell
            let b = if row == self.position.y && col == self.position.x {
                get_block()
                    .style(Style::default().fg(color).bg(color))
                    .border_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::SLOW_BLINK),
                    )
            } else {
                get_block()
                    .style(Style::default().fg(color).bg(color))
                    .border_style(Style::default().fg(Color::Green))
            };
            // let c = Paragraph::new("").block(b);
            b.render(cell, buf);
        }
    }
}
