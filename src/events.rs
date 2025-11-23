use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
use anyhow::Result;

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn poll(&self, timeout: Duration) -> Result<Option<KeyEvent>> {
        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                return Ok(Some(key_event));
            }
        }
        Ok(None)
    }
}
