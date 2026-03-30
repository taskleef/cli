use crate::client::ApiClient;
use crate::error::Result;
use crate::models::{BoardResponse, CardResponse, ColumnResponse, SubColumn, TodoResponse};
use futures::future::join_all;

/// A card with its resolved todo title
#[derive(Debug, Clone)]
pub struct ResolvedCard {
    pub card: CardResponse,
    pub todo: TodoResponse,
}

/// A column with all its resolved cards
#[derive(Debug, Clone)]
pub struct ResolvedColumn {
    pub column: ColumnResponse,
    pub cards: Vec<ResolvedCard>,
}

impl ResolvedColumn {
    pub fn cards_by_sub(&self, sub: &SubColumn) -> Vec<&ResolvedCard> {
        self.cards
            .iter()
            .filter(|c| c.card.sub_column.as_ref() == Some(sub))
            .collect()
    }

    /// Build a flat list of rows for this column: cards grouped by sub-column
    /// with section headers. Returns (rows, selectable_indices) where
    /// selectable_indices maps card cursor position -> row index.
    pub fn build_rows(&self) -> (Vec<ColumnRow>, Vec<usize>) {
        let mut rows = Vec::new();
        let mut selectable = Vec::new();

        let sections = [
            (SubColumn::Inbox, "Active"),
            (SubColumn::Blocked, "Blocked"),
            (SubColumn::Done, "Done"),
        ];

        for (sub, label) in &sections {
            let cards = self.cards_by_sub(sub);
            if cards.is_empty() {
                continue;
            }
            rows.push(ColumnRow::SectionHeader {
                label: label.to_string(),
                count: cards.len(),
                sub_column: sub.clone(),
            });
            for card in cards {
                selectable.push(rows.len());
                rows.push(ColumnRow::Card {
                    resolved: card.clone(),
                });
            }
        }

        (rows, selectable)
    }
}

/// A row in the column's flat list
#[derive(Debug, Clone)]
pub enum ColumnRow {
    SectionHeader {
        label: String,
        count: usize,
        sub_column: SubColumn,
    },
    Card {
        resolved: ResolvedCard,
    },
}

/// Source card info stashed when entering move mode
#[derive(Debug, Clone, PartialEq)]
pub struct MoveSource {
    pub card_id: String,
    pub todo_title: String,
    pub source_col: usize,
}

/// Snapshot of the card being viewed in detail
#[derive(Debug, Clone)]
pub struct CardDetail {
    pub todo: TodoResponse,
    pub card: CardResponse,
    pub column_title: String,
}

/// Animation state for the detail panel
#[derive(Debug, Clone)]
pub struct Animation {
    /// 0.0 = closed, 1.0 = fully open
    pub progress: f64,
    pub opening: bool,
    pub step: f64,
}

impl Animation {
    pub fn new_opening() -> Self {
        Self {
            progress: 0.0,
            opening: true,
            step: 0.15,
        }
    }

    pub fn new_closing(from: f64) -> Self {
        Self {
            progress: from,
            opening: false,
            step: 0.2,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.opening {
            self.progress = (self.progress + self.step).min(1.0);
            self.progress < 1.0
        } else {
            self.progress = (self.progress - self.step).max(0.0);
            self.progress > 0.0
        }
    }

    pub fn is_complete(&self) -> bool {
        if self.opening {
            self.progress >= 1.0
        } else {
            self.progress <= 0.0
        }
    }
}

/// What mode the board view is in
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    MoveSelect { source: MoveSource },
}

/// Which screen is active
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    BoardList,
    Board,
}

/// Full app state for the TUI
#[derive(Debug)]
pub struct App {
    pub screen: Screen,
    // Board list state
    pub boards: Vec<BoardResponse>,
    pub selected_board: usize,
    // Board view state
    pub board: Option<BoardResponse>,
    pub columns: Vec<ResolvedColumn>,
    pub selected_col: usize,
    pub selected_card: usize,
    pub mode: Mode,
    pub status_msg: String,
    pub should_quit: bool,
    pub detail: Option<CardDetail>,
    pub animation: Option<Animation>,
}

impl App {
    pub fn new_board_list(boards: Vec<BoardResponse>) -> Self {
        Self {
            screen: Screen::BoardList,
            boards,
            selected_board: 0,
            board: None,
            columns: Vec::new(),
            selected_col: 0,
            selected_card: 0,
            mode: Mode::Normal,
            status_msg: String::new(),
            should_quit: false,
            detail: None,
            animation: None,
        }
    }

    /// Set up the board view after loading board data
    pub fn enter_board(&mut self, board: BoardResponse, columns: Vec<ResolvedColumn>) {
        self.screen = Screen::Board;
        self.board = Some(board);
        self.columns = columns;
        self.selected_col = 0;
        self.selected_card = 0;
        self.mode = Mode::Normal;
        self.status_msg.clear();
        self.detail = None;
        self.animation = None;
    }

    /// Go back to board list
    pub fn back_to_board_list(&mut self) {
        self.screen = Screen::BoardList;
        self.board = None;
        self.columns.clear();
        self.selected_col = 0;
        self.selected_card = 0;
        self.mode = Mode::Normal;
        self.status_msg.clear();
        self.detail = None;
        self.animation = None;
    }

    /// Load board data from the API into an existing app
    pub async fn load_board(client: &dyn ApiClient, board_id: &str) -> Result<(BoardResponse, Vec<ResolvedColumn>)> {
        let board = client.get_board(board_id).await?;
        let mut columns = client.list_columns(board_id).await?;
        columns.sort_by_key(|c| c.order.unwrap_or(0));

        let card_futures: Vec<_> = columns
            .iter()
            .map(|col| {
                let col_id = col.id.clone();
                async move { client.list_cards(&col_id).await }
            })
            .collect();
        let card_results = join_all(card_futures).await;

        let mut resolved_columns = Vec::new();
        for (i, col) in columns.into_iter().enumerate() {
            let cards = match &card_results[i] {
                Ok(cards) => cards.clone(),
                Err(_) => Vec::new(),
            };

            let todo_futures: Vec<_> = cards
                .iter()
                .map(|card| {
                    let todo_id = card.todo_id.clone();
                    async move { client.get_todo(&todo_id).await }
                })
                .collect();
            let todo_results = join_all(todo_futures).await;

            let resolved_cards: Vec<ResolvedCard> = cards
                .into_iter()
                .zip(todo_results)
                .filter_map(|(card, todo_result)| {
                    let todo = todo_result.ok()?;
                    // Skip cards whose underlying todo is already completed —
                    // recurring todos leave completed cards on the board
                    if todo.is_completed == Some(true) {
                        return None;
                    }
                    Some(ResolvedCard { card, todo })
                })
                .collect();

            resolved_columns.push(ResolvedColumn {
                column: col,
                cards: resolved_cards,
            });
        }

        Ok((board, resolved_columns))
    }

    // --- Board list navigation ---

    pub fn board_list_down(&mut self) {
        if self.selected_board + 1 < self.boards.len() {
            self.selected_board += 1;
        }
    }

    pub fn board_list_up(&mut self) {
        if self.selected_board > 0 {
            self.selected_board -= 1;
        }
    }

    pub fn selected_board_id(&self) -> Option<&str> {
        self.boards.get(self.selected_board).map(|b| b.id.as_str())
    }

    // --- Board view navigation ---

    /// Number of selectable cards in the selected column (across all sub-columns)
    pub fn visible_card_count(&self) -> usize {
        self.columns
            .get(self.selected_col)
            .map(|col| col.build_rows().1.len())
            .unwrap_or(0)
    }

    /// Get the currently selected resolved card, if any
    pub fn selected_card_data(&self) -> Option<&ResolvedCard> {
        let col = self.columns.get(self.selected_col)?;
        let (rows, selectable) = col.build_rows();
        let row_idx = selectable.get(self.selected_card)?;
        match rows.get(*row_idx)? {
            ColumnRow::Card { resolved } => {
                // The resolved in build_rows is cloned, but we need a reference to the original.
                // Find it by card id.
                col.cards.iter().find(|c| c.card.id == resolved.card.id)
            }
            _ => None,
        }
    }

    pub fn move_left(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;
            self.clamp_card_selection();
        }
    }

    pub fn move_right(&mut self) {
        if self.selected_col + 1 < self.columns.len() {
            self.selected_col += 1;
            self.clamp_card_selection();
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_card > 0 {
            self.selected_card -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let count = self.visible_card_count();
        if count > 0 && self.selected_card + 1 < count {
            self.selected_card += 1;
        }
    }

    pub fn enter_move_mode(&mut self) {
        if let Some(card) = self.selected_card_data() {
            let source = MoveSource {
                card_id: card.card.id.clone(),
                todo_title: card.todo.title.clone(),
                source_col: self.selected_col,
            };
            self.status_msg = format!(
                "Moving '{}' — ←/→ to pick column, Enter to confirm, Esc to cancel",
                source.todo_title
            );
            self.mode = Mode::MoveSelect { source };
        }
    }

    pub fn cancel_mode(&mut self) {
        self.mode = Mode::Normal;
        self.status_msg.clear();
    }

    pub fn move_source(&self) -> Option<&MoveSource> {
        match &self.mode {
            Mode::MoveSelect { source } => Some(source),
            _ => None,
        }
    }

    pub fn is_move_mode(&self) -> bool {
        matches!(self.mode, Mode::MoveSelect { .. })
    }

    pub fn open_detail(&mut self) {
        if let Some(rc) = self.selected_card_data() {
            let column_title = self
                .columns
                .get(self.selected_col)
                .map(|c| c.column.title.clone())
                .unwrap_or_default();

            self.detail = Some(CardDetail {
                todo: rc.todo.clone(),
                card: rc.card.clone(),
                column_title,
            });
            self.animation = Some(Animation::new_opening());
        }
    }

    pub fn close_detail(&mut self) {
        if let Some(anim) = &self.animation {
            self.animation = Some(Animation::new_closing(anim.progress));
        } else if self.detail.is_some() {
            self.detail = None;
        }
    }

    pub fn tick_animation(&mut self) -> bool {
        if let Some(ref mut anim) = self.animation {
            let still_going = anim.tick();
            if anim.is_complete() && !anim.opening {
                self.detail = None;
                self.animation = None;
                return false;
            }
            if anim.is_complete() && anim.opening {
                self.animation = Some(Animation {
                    progress: 1.0,
                    opening: true,
                    step: anim.step,
                });
                return false;
            }
            still_going
        } else {
            false
        }
    }

    pub fn is_detail_visible(&self) -> bool {
        self.detail.is_some()
    }

    pub fn detail_progress(&self) -> f64 {
        self.animation
            .as_ref()
            .map(|a| a.progress)
            .unwrap_or(1.0)
    }

    fn clamp_card_selection(&mut self) {
        let count = self.visible_card_count();
        if count == 0 {
            self.selected_card = 0;
        } else if self.selected_card >= count {
            self.selected_card = count - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn make_todo(id: &str, title: &str) -> TodoResponse {
        TodoResponse {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            priority: None,
            due_date: None,
            is_completed: Some(false),
            subtasks: None,
            tags: None,
            assignee_user_id: None,
        }
    }

    fn make_resolved(id: &str, todo_id: &str, title: &str, sub: SubColumn) -> ResolvedCard {
        ResolvedCard {
            card: CardResponse {
                id: id.to_string(),
                todo_id: todo_id.to_string(),
                sub_column: Some(sub),
            },
            todo: make_todo(todo_id, title),
        }
    }

    fn make_test_app() -> App {
        let boards = vec![
            BoardResponse { id: "b1".into(), title: "Test Board".into() },
            BoardResponse { id: "b2".into(), title: "Other Board".into() },
        ];

        let columns = vec![
            ResolvedColumn {
                column: ColumnResponse {
                    id: "col1".into(), title: "Backlog".into(),
                    order: Some(0), wip_limit: None,
                },
                cards: vec![
                    make_resolved("c1", "t1", "Fix bug", SubColumn::Inbox),
                    make_resolved("c2", "t2", "Add tests", SubColumn::Inbox),
                    make_resolved("c3", "t3", "Done task", SubColumn::Done),
                    make_resolved("c5", "t5", "Stuck task", SubColumn::Blocked),
                ],
            },
            ResolvedColumn {
                column: ColumnResponse {
                    id: "col2".into(), title: "In Progress".into(),
                    order: Some(1), wip_limit: Some(3),
                },
                cards: vec![
                    make_resolved("c4", "t4", "Refactor auth", SubColumn::Inbox),
                ],
            },
            ResolvedColumn {
                column: ColumnResponse {
                    id: "col3".into(), title: "Done".into(),
                    order: Some(2), wip_limit: None,
                },
                cards: vec![],
            },
        ];

        let mut app = App::new_board_list(boards);
        app.enter_board(
            BoardResponse { id: "b1".into(), title: "Test Board".into() },
            columns,
        );
        app
    }

    // --- Board list tests ---

    #[test]
    fn test_board_list_initial() {
        let boards = vec![
            BoardResponse { id: "b1".into(), title: "Board 1".into() },
            BoardResponse { id: "b2".into(), title: "Board 2".into() },
        ];
        let app = App::new_board_list(boards);
        assert_eq!(app.screen, Screen::BoardList);
        assert_eq!(app.selected_board, 0);
        assert_eq!(app.selected_board_id(), Some("b1"));
    }

    #[test]
    fn test_board_list_navigate() {
        let boards = vec![
            BoardResponse { id: "b1".into(), title: "Board 1".into() },
            BoardResponse { id: "b2".into(), title: "Board 2".into() },
            BoardResponse { id: "b3".into(), title: "Board 3".into() },
        ];
        let mut app = App::new_board_list(boards);

        app.board_list_down();
        assert_eq!(app.selected_board, 1);
        assert_eq!(app.selected_board_id(), Some("b2"));

        app.board_list_down();
        assert_eq!(app.selected_board, 2);

        app.board_list_down(); // clamp
        assert_eq!(app.selected_board, 2);

        app.board_list_up();
        assert_eq!(app.selected_board, 1);

        app.board_list_up();
        app.board_list_up(); // clamp at 0
        assert_eq!(app.selected_board, 0);
    }

    #[test]
    fn test_enter_and_leave_board() {
        let boards = vec![
            BoardResponse { id: "b1".into(), title: "Board 1".into() },
        ];
        let mut app = App::new_board_list(boards);
        assert_eq!(app.screen, Screen::BoardList);

        app.enter_board(
            BoardResponse { id: "b1".into(), title: "Board 1".into() },
            vec![],
        );
        assert_eq!(app.screen, Screen::Board);
        assert!(app.board.is_some());

        app.back_to_board_list();
        assert_eq!(app.screen, Screen::BoardList);
        assert!(app.board.is_none());
        assert!(app.columns.is_empty());
    }

    // --- Column row building / sub-column navigation tests ---

    #[test]
    fn test_build_rows_groups_by_sub_column() {
        let col = ResolvedColumn {
            column: ColumnResponse {
                id: "c1".into(), title: "Col".into(), order: Some(0), wip_limit: None,
            },
            cards: vec![
                make_resolved("c1", "t1", "Active card", SubColumn::Inbox),
                make_resolved("c2", "t2", "Blocked card", SubColumn::Blocked),
                make_resolved("c3", "t3", "Done card", SubColumn::Done),
            ],
        };

        let (rows, selectable) = col.build_rows();

        // 3 section headers + 3 cards = 6 rows
        assert_eq!(rows.len(), 6);
        // 3 selectable cards
        assert_eq!(selectable.len(), 3);

        // Verify order: Active header, active card, Blocked header, blocked card, Done header, done card
        assert!(matches!(&rows[0], ColumnRow::SectionHeader { label, .. } if label == "Active"));
        assert!(matches!(&rows[1], ColumnRow::Card { resolved } if resolved.todo.title == "Active card"));
        assert!(matches!(&rows[2], ColumnRow::SectionHeader { label, .. } if label == "Blocked"));
        assert!(matches!(&rows[3], ColumnRow::Card { resolved } if resolved.todo.title == "Blocked card"));
        assert!(matches!(&rows[4], ColumnRow::SectionHeader { label, .. } if label == "Done"));
        assert!(matches!(&rows[5], ColumnRow::Card { resolved } if resolved.todo.title == "Done card"));
    }

    #[test]
    fn test_build_rows_skips_empty_sections() {
        let col = ResolvedColumn {
            column: ColumnResponse {
                id: "c1".into(), title: "Col".into(), order: Some(0), wip_limit: None,
            },
            cards: vec![
                make_resolved("c1", "t1", "Active card", SubColumn::Inbox),
            ],
        };

        let (rows, selectable) = col.build_rows();
        assert_eq!(rows.len(), 2); // 1 header + 1 card
        assert_eq!(selectable.len(), 1);
    }

    #[test]
    fn test_visible_card_count_includes_all_sub_columns() {
        let app = make_test_app();
        // Column 0: 2 inbox + 1 done + 1 blocked = 4 selectable cards
        assert_eq!(app.visible_card_count(), 4);
    }

    #[test]
    fn test_navigate_across_sub_columns() {
        let mut app = make_test_app();
        // Start at card 0 (first inbox card)
        assert_eq!(app.selected_card_data().unwrap().todo.title, "Fix bug");

        app.move_down();
        assert_eq!(app.selected_card_data().unwrap().todo.title, "Add tests");

        app.move_down();
        // Now on blocked card
        assert_eq!(app.selected_card_data().unwrap().todo.title, "Stuck task");

        app.move_down();
        // Now on done card
        assert_eq!(app.selected_card_data().unwrap().todo.title, "Done task");

        app.move_down(); // clamp
        assert_eq!(app.selected_card_data().unwrap().todo.title, "Done task");
    }

    // --- Existing board navigation tests ---

    #[test]
    fn test_initial_state() {
        let app = make_test_app();
        assert_eq!(app.selected_col, 0);
        assert_eq!(app.selected_card, 0);
        assert!(!app.is_move_mode());
        assert!(!app.should_quit);
    }

    #[test]
    fn test_selected_card_data() {
        let app = make_test_app();
        let card = app.selected_card_data().unwrap();
        assert_eq!(card.todo.title, "Fix bug");
    }

    #[test]
    fn test_move_right() {
        let mut app = make_test_app();
        app.move_right();
        assert_eq!(app.selected_col, 1);
        assert_eq!(app.selected_card, 0);
    }

    #[test]
    fn test_move_right_clamps_at_end() {
        let mut app = make_test_app();
        app.move_right();
        app.move_right();
        app.move_right();
        assert_eq!(app.selected_col, 2);
    }

    #[test]
    fn test_move_left() {
        let mut app = make_test_app();
        app.selected_col = 1;
        app.move_left();
        assert_eq!(app.selected_col, 0);
    }

    #[test]
    fn test_move_left_clamps_at_zero() {
        let mut app = make_test_app();
        app.move_left();
        assert_eq!(app.selected_col, 0);
    }

    #[test]
    fn test_move_down() {
        let mut app = make_test_app();
        app.move_down();
        assert_eq!(app.selected_card, 1);
    }

    #[test]
    fn test_move_up() {
        let mut app = make_test_app();
        app.selected_card = 1;
        app.move_up();
        assert_eq!(app.selected_card, 0);
    }

    #[test]
    fn test_move_up_clamps_at_zero() {
        let mut app = make_test_app();
        app.move_up();
        assert_eq!(app.selected_card, 0);
    }

    #[test]
    fn test_move_right_clamps_card_selection() {
        let mut app = make_test_app();
        app.selected_card = 3; // 4th card in backlog
        app.move_right(); // In Progress has 1 card
        assert_eq!(app.selected_card, 0);
    }

    #[test]
    fn test_move_right_to_empty_column() {
        let mut app = make_test_app();
        app.selected_col = 1;
        app.move_right();
        assert_eq!(app.selected_col, 2);
        assert_eq!(app.selected_card, 0);
        assert!(app.selected_card_data().is_none());
    }

    #[test]
    fn test_enter_move_mode() {
        let mut app = make_test_app();
        app.enter_move_mode();
        assert!(app.is_move_mode());
        assert!(!app.status_msg.is_empty());
        let source = app.move_source().unwrap();
        assert_eq!(source.card_id, "c1");
        assert_eq!(source.todo_title, "Fix bug");
        assert_eq!(source.source_col, 0);
    }

    #[test]
    fn test_enter_move_mode_no_card() {
        let mut app = make_test_app();
        app.selected_col = 2;
        app.enter_move_mode();
        assert!(!app.is_move_mode());
    }

    #[test]
    fn test_cancel_mode() {
        let mut app = make_test_app();
        app.enter_move_mode();
        app.cancel_mode();
        assert!(!app.is_move_mode());
        assert!(app.status_msg.is_empty());
    }

    #[test]
    fn test_move_mode_navigate_columns() {
        let mut app = make_test_app();
        app.enter_move_mode();
        assert!(app.is_move_mode());
        app.move_right();
        assert_eq!(app.selected_col, 1);
        let source = app.move_source().unwrap();
        assert_eq!(source.card_id, "c1");
        assert_eq!(source.source_col, 0);
    }

    // --- Card detail / animation tests ---

    #[test]
    fn test_open_detail() {
        let mut app = make_test_app();
        assert!(!app.is_detail_visible());

        app.open_detail();
        assert!(app.is_detail_visible());

        let detail = app.detail.as_ref().unwrap();
        assert_eq!(detail.todo.title, "Fix bug");
        assert_eq!(detail.column_title, "Backlog");
    }

    #[test]
    fn test_open_detail_on_empty_column_does_nothing() {
        let mut app = make_test_app();
        app.selected_col = 2;
        app.open_detail();
        assert!(!app.is_detail_visible());
    }

    #[test]
    fn test_open_detail_starts_animation() {
        let mut app = make_test_app();
        app.open_detail();
        let anim = app.animation.as_ref().unwrap();
        assert!(anim.opening);
        assert_eq!(anim.progress, 0.0);
    }

    #[test]
    fn test_animation_tick_advances_progress() {
        let mut app = make_test_app();
        app.open_detail();
        let still_going = app.tick_animation();
        assert!(still_going);
        let progress = app.detail_progress();
        assert!(progress > 0.0 && progress < 1.0);
    }

    #[test]
    fn test_animation_completes() {
        let mut app = make_test_app();
        app.open_detail();
        for _ in 0..20 {
            if !app.tick_animation() { break; }
        }
        assert!(app.detail_progress() >= 1.0);
        assert!(app.is_detail_visible());
    }

    #[test]
    fn test_close_detail_starts_closing_animation() {
        let mut app = make_test_app();
        app.open_detail();
        for _ in 0..20 {
            if !app.tick_animation() { break; }
        }
        app.close_detail();
        let anim = app.animation.as_ref().unwrap();
        assert!(!anim.opening);
        assert!(anim.progress > 0.0);
    }

    #[test]
    fn test_close_animation_clears_detail() {
        let mut app = make_test_app();
        app.open_detail();
        for _ in 0..20 {
            if !app.tick_animation() { break; }
        }
        app.close_detail();
        for _ in 0..20 {
            if !app.tick_animation() { break; }
        }
        assert!(!app.is_detail_visible());
        assert!(app.detail.is_none());
        assert!(app.animation.is_none());
    }

    #[test]
    fn test_detail_progress_defaults_to_1() {
        let app = make_test_app();
        assert_eq!(app.detail_progress(), 1.0);
    }

    #[test]
    fn test_animation_step_values() {
        let opening = Animation::new_opening();
        assert_eq!(opening.progress, 0.0);
        assert!(opening.opening);
        assert_eq!(opening.step, 0.15);

        let closing = Animation::new_closing(1.0);
        assert_eq!(closing.progress, 1.0);
        assert!(!closing.opening);
        assert_eq!(closing.step, 0.2);
    }

    #[test]
    fn test_close_during_open_animation() {
        let mut app = make_test_app();
        app.open_detail();
        app.tick_animation();
        let mid_progress = app.detail_progress();
        assert!(mid_progress > 0.0 && mid_progress < 1.0);
        app.close_detail();
        let anim = app.animation.as_ref().unwrap();
        assert!(!anim.opening);
        assert_eq!(anim.progress, mid_progress);
    }

    // --- Completed todo filtering tests ---

    fn make_completed_resolved(id: &str, todo_id: &str, title: &str, sub: SubColumn) -> ResolvedCard {
        ResolvedCard {
            card: CardResponse {
                id: id.to_string(),
                todo_id: todo_id.to_string(),
                sub_column: Some(sub),
            },
            todo: TodoResponse {
                id: todo_id.to_string(),
                title: title.to_string(),
                description: None,
                priority: None,
                due_date: None,
                is_completed: Some(true),
                subtasks: None,
                tags: None,
                assignee_user_id: None,
            },
        }
    }

    #[test]
    fn test_completed_todos_excluded_from_board() {
        // Simulates recurring todos: board has cards for both the completed
        // and the new incomplete version (different todo_ids, same title)
        let col = ResolvedColumn {
            column: ColumnResponse {
                id: "c1".into(), title: "Col".into(), order: Some(0), wip_limit: None,
            },
            cards: vec![
                make_resolved("card1", "t1-new", "Walk Obe", SubColumn::Inbox),
                make_completed_resolved("card2", "t1-old", "Walk Obe", SubColumn::Inbox),
                make_resolved("card3", "t2-new", "Get Money For Edith", SubColumn::Inbox),
                make_completed_resolved("card4", "t2-old", "Get Money For Edith", SubColumn::Inbox),
                make_resolved("card5", "t3", "Unique task", SubColumn::Inbox),
            ],
        };

        // build_rows doesn't filter — the filtering happens at load time.
        // But we can verify that if only incomplete cards are present, no dupes.
        let incomplete_col = ResolvedColumn {
            column: col.column.clone(),
            cards: col.cards.into_iter()
                .filter(|c| c.todo.is_completed != Some(true))
                .collect(),
        };

        let (rows, selectable) = incomplete_col.build_rows();
        assert_eq!(selectable.len(), 3);

        let card_titles: Vec<&str> = rows.iter().filter_map(|r| match r {
            ColumnRow::Card { resolved } => Some(resolved.todo.title.as_str()),
            _ => None,
        }).collect();
        assert_eq!(card_titles, vec!["Walk Obe", "Get Money For Edith", "Unique task"]);
    }

    // --- Detail on blocked card ---

    #[test]
    fn test_open_detail_on_blocked_card() {
        let mut app = make_test_app();
        // Navigate to the blocked card (index 2 in the selectable list)
        app.selected_card = 2;
        let card = app.selected_card_data().unwrap();
        assert_eq!(card.todo.title, "Stuck task");

        app.open_detail();
        assert!(app.is_detail_visible());
        assert_eq!(app.detail.as_ref().unwrap().todo.title, "Stuck task");
    }
}
