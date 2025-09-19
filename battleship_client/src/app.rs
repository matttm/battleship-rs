use std::collections::VecDeque;

use crate::{
    event::{AppEvent, Event, EventHandler},
    widgets::notification_pane::NotificationPane,
};
use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    symbols::scrollbar,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    pub notification_pane: NotificationPane,
    tx: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://"))
            .await
            .expect("Cannot connect to game server");
        let (tx, rx) = socket.split();
        let d = Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            notification_pane: NotificationPane::new(VecDeque::new()),
            tx,
            rx,
        };
        d
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| {
                self.render_app(frame);
            })?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }
    fn render_app(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(20)])
            .split(frame.area());
        frame.render_widget(self, layout[0]);
        frame.render_widget(&self.notification_pane, layout[1]);
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }
}
