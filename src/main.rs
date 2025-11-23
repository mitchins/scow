mod app;
mod config;
mod events;
mod scanner;
mod ui;
mod worker;

use app::{AppMode, AppState};
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::EventHandler;
use ratatui::{backend::CrosstermBackend, Terminal};
use scanner::Scanner;
use std::env;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;

fn main() -> Result<()> {
    // Get the directory to scan from args or use current directory
    let current_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Load config
    let config = config::load_config().unwrap_or_default();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = AppState::new(current_path.clone(), config);

    // Start with scanning mode
    app.mode = AppMode::Scanning;

    // Scan in background (for MVP, we'll do it synchronously)
    match Scanner::scan(&current_path) {
        Ok(root) => {
            app.root_node = Some(root);
            app.mode = AppMode::Browsing;
        }
        Err(e) => {
            eprintln!("Failed to scan: {}", e);
            app.mode = AppMode::Browsing;
        }
    }

    // Event handler
    let event_handler = EventHandler::new();

    // Main loop
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if app.should_quit {
            break;
        }

        // Handle events
        if let Some(key_event) = event_handler.poll(Duration::from_millis(100))? {
            match app.mode {
                AppMode::Browsing => handle_browsing_input(&mut app, key_event.code),
                AppMode::DestinationPicker => handle_destination_picker_input(&mut app, key_event.code),
                AppMode::Confirming => handle_confirmation_input(&mut app, key_event.code),
                AppMode::Processing => {
                    // Wait for processing to complete
                }
                AppMode::Scanning => {
                    // Wait for scan to complete
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_browsing_input(app: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('s') => {
            // Rescan
            app.mode = AppMode::Scanning;
            if let Ok(root) = Scanner::scan(&app.current_path) {
                app.root_node = Some(root);
            }
            app.mode = AppMode::Browsing;
        }
        KeyCode::Char(' ') => {
            // Mark for delete (simplified - marks the current path)
            if let Some(ref root) = app.root_node {
                app.staging.toggle_delete(root.path.clone(), root.size);
            }
        }
        KeyCode::Char('m') => {
            // Open destination picker
            app.mode = AppMode::DestinationPicker;
        }
        KeyCode::Enter => {
            // Commit operations
            if !app.staging.ops.is_empty() {
                app.mode = AppMode::Confirming;
            }
        }
        KeyCode::Char('c') => {
            // Clear staging
            app.staging.clear();
        }
        _ => {}
    }
}

fn handle_destination_picker_input(app: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Browsing;
        }
        KeyCode::Enter => {
            // Select destination (simplified - use first destination)
            if let Some(dest) = app.config.destinations.first() {
                if let Some(ref root) = app.root_node {
                    app.staging.set_move(
                        root.path.clone(),
                        PathBuf::from(&dest.path),
                    );
                }
            }
            app.mode = AppMode::Browsing;
        }
        _ => {}
    }
}

fn handle_confirmation_input(app: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('y') => {
            // Execute operations
            app.mode = AppMode::Processing;
            
            let ops: Vec<_> = app.staging.ops.iter()
                .map(|(path, action)| (path.clone(), action.clone()))
                .collect();
            
            if let Err(e) = worker::Worker::execute(ops) {
                eprintln!("Error executing operations: {}", e);
            }
            
            app.staging.clear();
            
            // Rescan
            if let Ok(root) = Scanner::scan(&app.current_path) {
                app.root_node = Some(root);
            }
            
            app.mode = AppMode::Browsing;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = AppMode::Browsing;
        }
        _ => {}
    }
}
