use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Padding, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

pub fn draw(f: &mut Frame, app: &App, simple: bool, theme: &Theme) {
    if simple {
        draw_simple(f, app);
    } else {
        draw_rich(f, app, theme);
    }
}

fn sel_suffix(app: &App) -> String {
    if app.multi() {
        format!(" · {} sel", app.selected_count())
    } else {
        String::new()
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
        "{} ({}/{}{})",
        app.prompt(),
        app.cursor() + 1,
        app.items().len(),
        sel_suffix(app),
    );
    f.render_widget(Paragraph::new(header), header_area);

    let items: Vec<ListItem> = app.items()[visible.clone()]
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if app.multi() {
                let mark = if app.selected()[visible.start + i] {
                    "[x] "
                } else {
                    "[ ] "
                };
                ListItem::new(format!("{mark}{s}"))
            } else {
                ListItem::new(s.as_str())
            }
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.cursor().saturating_sub(visible.start)));
    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, body_area, &mut state);
}

fn draw_rich(f: &mut Frame, app: &App, theme: &Theme) {
    let visible = app.visible_range();
    let overflow = app.items().len() > visible.len();
    let indicator_rows: u16 = if overflow { 2 } else { 0 };
    let list_rows = visible.len() as u16 + indicator_rows;
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
            Style::default()
                .fg(theme.border)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]);
    let title_right = Line::from(Span::styled(
        format!(
            " {}/{}{} ",
            app.cursor() + 1,
            app.items().len(),
            sel_suffix(app),
        ),
        Style::default().add_modifier(Modifier::DIM),
    ))
    .right_aligned();

    let hidden_above = visible.start;
    let hidden_below = app.items().len().saturating_sub(visible.end);

    let v_pad: u16 = if overflow { 0 } else { 1 };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .padding(Padding::new(1, 1, v_pad, v_pad))
        .title(title_left)
        .title(title_right);

    let cursor_local = app.cursor().saturating_sub(visible.start);
    let dim = Style::default().add_modifier(Modifier::DIM);
    let cursor_style = Style::default()
        .fg(theme.cursor)
        .add_modifier(Modifier::BOLD);
    let plain = Style::default();
    let mut items: Vec<ListItem> = Vec::with_capacity(visible.len() + indicator_rows as usize);
    if overflow {
        let text = if hidden_above > 0 {
            format!("  ↑ {hidden_above} more")
        } else {
            String::new()
        };
        items.push(ListItem::new(Line::from(Span::styled(text, dim))));
    }
    items.extend(
        app.items()[visible.clone()]
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let is_cursor = i == cursor_local;
                let mut spans: Vec<Span> = Vec::with_capacity(4);
                if is_cursor {
                    spans.push(Span::styled("❯ ", cursor_style));
                } else {
                    spans.push(Span::raw("  "));
                }
                if app.multi() {
                    let on = app.selected()[visible.start + i];
                    let mark = if on { "[x] " } else { "[ ] " };
                    spans.push(Span::styled(mark, if on { cursor_style } else { dim }));
                }
                spans.push(Span::styled(
                    s.as_str(),
                    if is_cursor { cursor_style } else { plain },
                ));
                ListItem::new(Line::from(spans))
            }),
    );
    if overflow {
        let text = if hidden_below > 0 {
            format!("  ↓ {hidden_below} more")
        } else {
            String::new()
        };
        items.push(ListItem::new(Line::from(Span::styled(text, dim))));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, box_area);

    let mut hint_spans: Vec<Span> = vec![
        Span::raw("  "),
        Span::styled("↑/↓", dim),
        Span::styled(" move · ", dim),
    ];
    if app.multi() {
        hint_spans.push(Span::styled("[space]", dim));
        hint_spans.push(Span::styled(" toggle · ", dim));
        hint_spans.push(Span::styled("[Ctrl + a]", dim));
        hint_spans.push(Span::styled(" select all · ", dim));
        hint_spans.push(Span::styled("[Ctrl + d]", dim));
        hint_spans.push(Span::styled(" all clear · ", dim));
    }
    hint_spans.push(Span::styled("↵", dim));
    let confirm_label = if app.multi() {
        " confirm selected · "
    } else {
        " confirm · "
    };
    hint_spans.push(Span::styled(confirm_label, dim));
    hint_spans.push(Span::styled("[esc]", dim));
    hint_spans.push(Span::styled(" cancel", dim));
    f.render_widget(Paragraph::new(Line::from(hint_spans)), footer_area);
}
