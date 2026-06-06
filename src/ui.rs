use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
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
