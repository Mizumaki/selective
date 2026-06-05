use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Confirm(Vec<String>),
    Cancel,
}

pub struct App {
    items: Vec<String>,
    cursor: usize,
    offset: usize,
    prompt: String,
    height: u16,
    multi: bool,
    selected: Vec<bool>,
}

impl App {
    pub fn new(items: Vec<String>, prompt: String, height: u16, multi: bool) -> Self {
        let selected = vec![false; items.len()];
        Self { items, cursor: 0, offset: 0, prompt, height, multi, selected }
    }

    pub fn items(&self) -> &[String] { &self.items }
    pub fn cursor(&self) -> usize { self.cursor }
    pub fn prompt(&self) -> &str { &self.prompt }
    pub fn multi(&self) -> bool { self.multi }
    pub fn selected(&self) -> &[bool] { &self.selected }
    pub fn selected_count(&self) -> usize { self.selected.iter().filter(|&&b| b).count() }

    pub fn visible_range(&self) -> std::ops::Range<usize> {
        let h = self.height.max(1) as usize;
        let end = (self.offset + h).min(self.items.len());
        self.offset..end
    }

    pub fn set_height(&mut self, height: u16) {
        self.height = height;
        self.clamp_offset();
    }

    fn clamp_offset(&mut self) {
        let h = self.height.max(1) as usize;
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + h {
            self.offset = self.cursor + 1 - h;
        }
    }

    fn confirmed_lines(&self) -> Option<Vec<String>> {
        if self.multi {
            let lines: Vec<String> = self
                .items
                .iter()
                .zip(self.selected.iter())
                .filter(|&(_, &on)| on)
                .map(|(s, _)| s.clone())
                .collect();
            Some(lines).filter(|v| !v.is_empty())
        } else {
            Some(vec![self.items[self.cursor].clone()])
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Action::Cancel;
        }
        if self.multi && key.modifiers.is_empty() && key.code == KeyCode::Char(' ') {
            self.selected[self.cursor] = !self.selected[self.cursor];
            return Action::Continue;
        }
        if self.multi && key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('a') => {
                    self.selected.fill(true);
                    return Action::Continue;
                }
                KeyCode::Char('d') | KeyCode::Char('u') => {
                    self.selected.fill(false);
                    return Action::Continue;
                }
                _ => {}
            }
        }
        let action = match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return Action::Cancel,
            KeyCode::Enter => {
                return self
                    .confirmed_lines()
                    .map_or(Action::Continue, Action::Confirm);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < self.items.len() {
                    self.cursor += 1;
                }
                Action::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = self.cursor.saturating_sub(1);
                Action::Continue
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.cursor = 0;
                Action::Continue
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.cursor = self.items.len().saturating_sub(1);
                Action::Continue
            }
            KeyCode::PageDown => {
                let step = self.height.max(1) as usize;
                let last = self.items.len().saturating_sub(1);
                self.cursor = self.cursor.saturating_add(step).min(last);
                Action::Continue
            }
            KeyCode::PageUp => {
                let step = self.height.max(1) as usize;
                self.cursor = self.cursor.saturating_sub(step);
                Action::Continue
            }
            _ => Action::Continue,
        };
        self.clamp_offset();
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn app(items: &[&str]) -> App {
        App::new(items.iter().map(|s| s.to_string()).collect(), "".into(), 10, false)
    }

    fn multi_app(items: &[&str]) -> App {
        App::new(items.iter().map(|s| s.to_string()).collect(), "".into(), 10, true)
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn ctrl_a_selects_all_in_multi_mode() {
        let mut a = multi_app(&["a", "b", "c"]);
        assert_eq!(a.handle_key(ctrl(KeyCode::Char('a'))), Action::Continue);
        assert_eq!(
            a.handle_key(key(KeyCode::Enter)),
            Action::Confirm(vec!["a".into(), "b".into(), "c".into()])
        );
    }

    #[test]
    fn enter_with_no_selections_in_multi_mode_does_not_confirm() {
        let mut a = multi_app(&["a", "b", "c"]);
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Continue);
    }

    #[test]
    fn ctrl_d_clears_all_selections_in_multi_mode() {
        let mut a = multi_app(&["a", "b", "c"]);
        a.handle_key(ctrl(KeyCode::Char('a')));
        assert_eq!(a.handle_key(ctrl(KeyCode::Char('d'))), Action::Continue);
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Continue);
    }

    #[test]
    fn ctrl_u_clears_all_selections_in_multi_mode() {
        let mut a = multi_app(&["a", "b", "c"]);
        a.handle_key(ctrl(KeyCode::Char('a')));
        assert_eq!(a.handle_key(ctrl(KeyCode::Char('u'))), Action::Continue);
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Continue);
    }

    #[test]
    fn ctrl_a_is_noop_in_single_mode() {
        let mut a = app(&["alpha", "beta"]);
        assert_eq!(a.handle_key(ctrl(KeyCode::Char('a'))), Action::Continue);
        assert_eq!(
            a.handle_key(key(KeyCode::Enter)),
            Action::Confirm(vec!["alpha".into()])
        );
    }

    #[test]
    fn ctrl_d_and_ctrl_u_are_noop_in_single_mode() {
        let mut a = app(&["alpha", "beta"]);
        assert_eq!(a.handle_key(ctrl(KeyCode::Char('d'))), Action::Continue);
        assert_eq!(a.handle_key(ctrl(KeyCode::Char('u'))), Action::Continue);
        assert_eq!(
            a.handle_key(key(KeyCode::Enter)),
            Action::Confirm(vec!["alpha".into()])
        );
    }

    #[test]
    fn space_is_noop_in_single_mode() {
        let mut a = app(&["alpha", "beta"]);
        assert_eq!(a.handle_key(key(KeyCode::Char(' '))), Action::Continue);
        assert_eq!(
            a.handle_key(key(KeyCode::Enter)),
            Action::Confirm(vec!["alpha".into()])
        );
    }

    #[test]
    fn enter_in_multi_mode_returns_selected_items_in_input_order() {
        let mut a = multi_app(&["a", "b", "c", "d"]);
        // toggle index 2 first
        a.handle_key(key(KeyCode::Down));
        a.handle_key(key(KeyCode::Down));
        a.handle_key(key(KeyCode::Char(' ')));
        // then toggle index 0
        a.handle_key(key(KeyCode::Home));
        a.handle_key(key(KeyCode::Char(' ')));
        assert_eq!(
            a.handle_key(key(KeyCode::Enter)),
            Action::Confirm(vec!["a".into(), "c".into()])
        );
    }

    #[test]
    fn space_toggles_selection_in_multi_mode() {
        let mut a = multi_app(&["alpha", "beta", "gamma"]);
        a.handle_key(key(KeyCode::Down));
        assert_eq!(a.handle_key(key(KeyCode::Char(' '))), Action::Continue);
        assert_eq!(
            a.handle_key(key(KeyCode::Enter)),
            Action::Confirm(vec!["beta".into()])
        );
    }

    #[test]
    fn enter_confirms_current_item() {
        let mut a = app(&["alpha", "beta", "gamma"]);
        assert_eq!(
            a.handle_key(key(KeyCode::Enter)),
            Action::Confirm(vec!["alpha".into()])
        );
    }

    #[test]
    fn visible_range_scrolls_when_cursor_leaves_viewport() {
        let items: Vec<String> = (0..10).map(|i| i.to_string()).collect();
        let mut a = App::new(items, "".into(), 3, false);
        assert_eq!(a.visible_range(), 0..3);
        for _ in 0..3 { a.handle_key(key(KeyCode::Down)); }
        assert_eq!(a.cursor(), 3);
        let r = a.visible_range();
        assert!(r.contains(&3), "cursor 3 must be visible, got {r:?}");
        assert_eq!(r.end - r.start, 3);
    }

    #[test]
    fn visible_range_scrolls_back_when_cursor_moves_up() {
        let items: Vec<String> = (0..10).map(|i| i.to_string()).collect();
        let mut a = App::new(items, "".into(), 3, false);
        a.handle_key(key(KeyCode::End));
        let r = a.visible_range();
        assert!(r.contains(&9));
        for _ in 0..9 { a.handle_key(key(KeyCode::Up)); }
        assert_eq!(a.cursor(), 0);
        assert_eq!(a.visible_range(), 0..3);
    }

    #[test]
    fn pagedown_jumps_by_viewport_height_pageup_back() {
        let items: Vec<String> = (0..20).map(|i| i.to_string()).collect();
        let mut a = App::new(items, "".into(), 5, false);
        a.handle_key(key(KeyCode::PageDown));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["5".into()]));
        let items: Vec<String> = (0..20).map(|i| i.to_string()).collect();
        let mut a = App::new(items, "".into(), 5, false);
        a.handle_key(key(KeyCode::PageDown));
        a.handle_key(key(KeyCode::PageDown));
        a.handle_key(key(KeyCode::PageUp));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["5".into()]));
    }

    #[test]
    fn pagedown_clamps_to_last() {
        let items: Vec<String> = (0..4).map(|i| i.to_string()).collect();
        let mut a = App::new(items, "".into(), 10, false);
        a.handle_key(key(KeyCode::PageDown));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["3".into()]));
    }

    #[test]
    fn home_goes_to_first_end_goes_to_last() {
        let mut a = app(&["a", "b", "c"]);
        a.handle_key(key(KeyCode::End));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["c".into()]));
        let mut a = app(&["a", "b", "c"]);
        a.handle_key(key(KeyCode::End));
        a.handle_key(key(KeyCode::Home));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["a".into()]));
    }

    #[test]
    fn g_goes_to_first_capital_g_goes_to_last() {
        let mut a = app(&["a", "b", "c"]);
        a.handle_key(key(KeyCode::Char('G')));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["c".into()]));
        let mut a = app(&["a", "b", "c"]);
        a.handle_key(key(KeyCode::Char('G')));
        a.handle_key(key(KeyCode::Char('g')));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["a".into()]));
    }

    #[test]
    fn j_moves_down_k_moves_up() {
        let mut a = app(&["alpha", "beta", "gamma"]);
        a.handle_key(key(KeyCode::Char('j')));
        a.handle_key(key(KeyCode::Char('j')));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["gamma".into()]));
        let mut a = app(&["alpha", "beta", "gamma"]);
        a.handle_key(key(KeyCode::Char('j')));
        a.handle_key(key(KeyCode::Char('k')));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["alpha".into()]));
    }

    #[test]
    fn esc_cancels() {
        let mut a = app(&["alpha"]);
        assert_eq!(a.handle_key(key(KeyCode::Esc)), Action::Cancel);
    }

    #[test]
    fn ctrl_c_cancels() {
        let mut a = app(&["alpha"]);
        let ev = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(a.handle_key(ev), Action::Cancel);
    }

    #[test]
    fn q_cancels() {
        let mut a = app(&["alpha"]);
        assert_eq!(a.handle_key(key(KeyCode::Char('q'))), Action::Cancel);
    }

    #[test]
    fn plain_c_does_not_cancel() {
        let mut a = app(&["alpha"]);
        assert_eq!(a.handle_key(key(KeyCode::Char('c'))), Action::Continue);
    }

    #[test]
    fn up_clamps_at_first_item() {
        let mut a = app(&["alpha", "beta"]);
        a.handle_key(key(KeyCode::Down));
        a.handle_key(key(KeyCode::Up));
        a.handle_key(key(KeyCode::Up));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["alpha".into()]));
    }

    #[test]
    fn down_clamps_at_last_item() {
        let mut a = app(&["alpha", "beta"]);
        a.handle_key(key(KeyCode::Down));
        a.handle_key(key(KeyCode::Down));
        a.handle_key(key(KeyCode::Down));
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["beta".into()]));
    }

    #[test]
    fn down_then_enter_confirms_next_item() {
        let mut a = app(&["alpha", "beta", "gamma"]);
        assert_eq!(a.handle_key(key(KeyCode::Down)), Action::Continue);
        assert_eq!(a.handle_key(key(KeyCode::Enter)), Action::Confirm(vec!["beta".into()]));
    }
}
