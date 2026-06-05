use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, ListState, Padding, Paragraph,
};

use crate::app::App;

const ACCENT: Color = Color::Cyan;

pub fn draw(f: &mut Frame, app: &App, simple: bool) {
    if simple {
        draw_simple(f, app);
    } else {
        draw_rich(f, app);
    }
}

fn draw_simple(f: &mut Frame, app: &App) {
    let visible = app.visible_range();
    let list_rows = visible.len() as u16;

    let [header_area, body_area, _rest] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(list_rows.max(1)),
        Constraint::Min(0),
    ])
    .areas(f.area());

    let header = format!(
        "{} ({}/{})",
        app.prompt(),
        app.cursor() + 1,
        app.items().len()
    );
    f.render_widget(Paragraph::new(header), header_area);

    let items: Vec<ListItem> = app.items()[visible.clone()]
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.cursor().saturating_sub(visible.start)));
    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, body_area, &mut state);
}

fn draw_rich(f: &mut Frame, app: &App) {
    let visible = app.visible_range();
    let list_rows = visible.len() as u16;
    let box_rows = list_rows.saturating_add(4).max(5);

    let [_gap_top, box_area, footer_area, _gap_bottom, _rest] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(box_rows),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(f.area());

    let title_left = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            app.prompt().to_string(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]);
    let title_right = Line::from(Span::styled(
        format!(" {}/{} ", app.cursor() + 1, app.items().len()),
        Style::default().add_modifier(Modifier::DIM),
    ))
    .right_aligned();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .padding(Padding::new(1, 1, 1, 1))
        .title(title_left)
        .title(title_right);

    let cursor_local = app.cursor().saturating_sub(visible.start);
    let items: Vec<ListItem> = app.items()[visible.clone()]
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i == cursor_local {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        "❯ ",
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        s.as_str(),
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    ),
                ]))
            } else {
                ListItem::new(Line::from(vec![Span::raw("  "), Span::raw(s.as_str())]))
            }
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, box_area);

    let hint = Line::from(vec![
        Span::raw("  "),
        Span::styled("↑/↓", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(" move · ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled("↵", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(" confirm · ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled("esc", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(" cancel", Style::default().add_modifier(Modifier::DIM)),
    ]);
    f.render_widget(Paragraph::new(hint), footer_area);
}
