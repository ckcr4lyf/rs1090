use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use crossterm::{execute, terminal::*};
use futures::{FutureExt, StreamExt};
use ratatui::prelude::*;
use std::io::{self, stderr, Stderr};
use tokio::sync::mpsc;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stderr>>;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stderr(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stderr()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Key(KeyEvent),
    Error,
    Tick,
}

#[derive(Debug)]
pub struct EventHandler {
    _tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    //task: Option<JoinHandle<()>>,
}

impl EventHandler {
    pub fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(250);

        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        let _task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  maybe_event = crossterm_event => {
                    match maybe_event {
                      Some(Ok(evt)) => {
                        match evt {
                          crossterm::event::Event::Key(key) => {
                            if key.kind == crossterm::event::KeyEventKind::Press {
                              tx.send(Event::Key(key)).unwrap();
                            }
                          },
                          _ => {},
                        }
                      }
                      Some(Err(_)) => {
                        tx.send(Event::Error).unwrap();
                      }
                      None => {},
                    }
                  },
                  _ = delay => {
                      tx.send(Event::Tick).unwrap();
                  },
                }
            }
        });

        Self {
            _tx,
            rx,
            //task: Some(_task),
        }
    }

    pub async fn next(&mut self) -> Result<Event, color_eyre::eyre::Error> {
        self.rx
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("Unable to get event"))
    }
}
