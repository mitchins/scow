use crate::app::{AppState, AppMode, Action};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[0]);

    // Left pane - Tree View
    draw_tree_view(f, app, main_chunks[0]);

    // Right pane - Staging View
    draw_staging_view(f, app, main_chunks[1]);

    // Footer
    draw_footer(f, app, chunks[1]);

    // Modal overlays
    match app.mode {
        AppMode::DestinationPicker => draw_destination_picker(f, app),
        AppMode::Confirming => draw_confirmation(f, app),
        AppMode::Processing => draw_processing(f, app),
        _ => {}
    }
}

fn draw_tree_view(f: &mut Frame, app: &AppState, area: Rect) {
    let block = Block::default()
        .title("File Tree")
        .borders(Borders::ALL);

    if let Some(ref root) = app.root_node {
        let items: Vec<ListItem> = build_tree_items(root, &app.staging.ops, 0);
        let list = List::new(items).block(block);
        f.render_widget(list, area);
    } else {
        let text = match app.mode {
            AppMode::Scanning => "Scanning...",
            _ => "No files loaded. Press 's' to scan.",
        };
        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }
}

fn build_tree_items(
    node: &crate::app::FileNode,
    staging_ops: &std::collections::HashMap<std::path::PathBuf, Action>,
    depth: usize,
) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    let indent = "  ".repeat(depth);
    let icon = if node.is_dir { "📁" } else { "📄" };
    
    let name = node.path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    
    let size_str = human_bytes::human_bytes(node.size as f64);
    
    let mut spans = vec![
        Span::raw(indent),
        Span::raw(format!("{} ", icon)),
    ];

    let (text_span, suffix) = match staging_ops.get(&node.path) {
        Some(Action::Delete) => (
            Span::styled(
                name.clone(),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::CROSSED_OUT),
            ),
            Some(Span::styled(" [DELETE]", Style::default().fg(Color::Red))),
        ),
        Some(Action::Move(dest)) => (
            Span::styled(name.clone(), Style::default().fg(Color::Yellow)),
            Some(Span::styled(
                format!(" ➜ {}", dest.display()),
                Style::default().fg(Color::Yellow),
            )),
        ),
        None => (Span::raw(name.clone()), None),
    };

    spans.push(text_span);
    spans.push(Span::raw(format!(" ({})", size_str)));
    
    if let Some(suffix_span) = suffix {
        spans.push(suffix_span);
    }

    items.push(ListItem::new(Line::from(spans)));

    // Recursively add children
    for child in &node.children {
        items.extend(build_tree_items(child, staging_ops, depth + 1));
    }

    items
}

fn draw_staging_view(f: &mut Frame, app: &AppState, area: Rect) {
    let block = Block::default()
        .title("Staging Area")
        .borders(Borders::ALL);

    let mut items = Vec::new();
    
    for (path, action) in &app.staging.ops {
        let action_str = match action {
            Action::Delete => format!("DELETE: {}", path.display()),
            Action::Move(dest) => format!("MOVE: {} → {}", path.display(), dest.display()),
        };
        items.push(ListItem::new(action_str));
    }

    let net_change_str = if app.staging.net_size_change < 0 {
        format!("Space to free: {}", human_bytes::human_bytes(-app.staging.net_size_change as f64))
    } else if app.staging.net_size_change > 0 {
        format!("Space to use: {}", human_bytes::human_bytes(app.staging.net_size_change as f64))
    } else {
        "No net change".to_string()
    };

    items.push(ListItem::new(""));
    items.push(ListItem::new(net_change_str));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, _app: &AppState, area: Rect) {
    let help_text = Line::from(vec![
        Span::raw("q: Quit | "),
        Span::raw("s: Scan | "),
        Span::raw("Space: Mark Delete | "),
        Span::raw("m: Move | "),
        Span::raw("Enter: Commit"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let paragraph = Paragraph::new(help_text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_destination_picker(f: &mut Frame, app: &AppState) {
    let area = centered_rect(60, 50, f.area());
    let block = Block::default()
        .title("Select Destination")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let items: Vec<ListItem> = app
        .config
        .destinations
        .iter()
        .map(|d| ListItem::new(format!("{}: {}", d.name, d.path)))
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_confirmation(f: &mut Frame, app: &AppState) {
    let area = centered_rect(50, 30, f.area());
    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let text = format!(
        "Execute {} operations?\n\nPress 'y' to confirm, 'n' to cancel",
        app.staging.ops.len()
    );
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_processing(f: &mut Frame, _app: &AppState) {
    let area = centered_rect(40, 20, f.area());
    let block = Block::default()
        .title("Processing")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new("Processing operations...").block(block);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
