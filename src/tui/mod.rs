pub mod app;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

use crate::client::ApiClient;
use crate::error::Result;

use app::{App, Mode, Screen};

pub async fn run(client: &dyn ApiClient) -> Result<()> {
    let boards = client.list_boards().await?;
    let mut app = App::new_board_list(boards);

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app, client).await;

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    client: &dyn ApiClient,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        let poll_ms = if app.animation.as_ref().is_some_and(|a| !a.is_complete()) {
            16
        } else {
            100
        };

        app.tick_animation();

        if event::poll(Duration::from_millis(poll_ms))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.screen {
                    Screen::BoardList => {
                        match key.code {
                            KeyCode::Char('q') => {
                                app.should_quit = true;
                                break;
                            }
                            KeyCode::Up | KeyCode::Char('k') => app.board_list_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.board_list_down(),
                            KeyCode::Enter => {
                                if let Some(board_id) = app.selected_board_id().map(|s| s.to_string()) {
                                    app.status_msg = "Loading board...".into();
                                    terminal.draw(|frame| ui::draw(frame, app))?;

                                    let (board, columns) = App::load_board(client, &board_id).await?;
                                    app.enter_board(board, columns);
                                }
                            }
                            _ => {}
                        }
                    }
                    Screen::Board => {
                        // Detail view intercepts all keys
                        if app.is_detail_visible() {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                                    app.close_detail();
                                }
                                _ => {}
                            }
                            continue;
                        }

                        match app.mode {
                            Mode::Normal => match key.code {
                                KeyCode::Char('q') => {
                                    app.should_quit = true;
                                    break;
                                }
                                KeyCode::Esc => app.back_to_board_list(),
                                KeyCode::Left | KeyCode::Char('h') => app.move_left(),
                                KeyCode::Right | KeyCode::Char('l') => app.move_right(),
                                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                                KeyCode::Enter => app.open_detail(),
                                KeyCode::Char('m') => app.enter_move_mode(),
                                KeyCode::Char('d') => {
                                    handle_set_sub_column(app, client, "Done").await?;
                                    reload_board(app, client).await?;
                                }
                                KeyCode::Char('b') => {
                                    handle_set_sub_column(app, client, "Blocked").await?;
                                    reload_board(app, client).await?;
                                }
                                KeyCode::Char('i') => {
                                    handle_set_sub_column(app, client, "Inbox").await?;
                                    reload_board(app, client).await?;
                                }
                                KeyCode::Char('r') => {
                                    reload_board(app, client).await?;
                                    app.status_msg = "Refreshed".into();
                                }
                                _ => {}
                            },
                            Mode::MoveSelect { .. } => match key.code {
                                KeyCode::Left | KeyCode::Char('h') => app.move_left(),
                                KeyCode::Right | KeyCode::Char('l') => app.move_right(),
                                KeyCode::Enter => {
                                    handle_move(app, client).await?;
                                    reload_board(app, client).await?;
                                }
                                KeyCode::Esc | KeyCode::Char('q') => app.cancel_mode(),
                                _ => {}
                            },
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_move(app: &mut App, client: &dyn ApiClient) -> Result<()> {
    let source = match app.move_source() {
        Some(s) => s.clone(),
        None => {
            app.cancel_mode();
            return Ok(());
        }
    };

    let target_col = match app.columns.get(app.selected_col) {
        Some(col) => col.column.id.clone(),
        None => {
            app.cancel_mode();
            return Ok(());
        }
    };

    client
        .update_card(
            &source.card_id,
            serde_json::json!({"columnId": target_col, "subColumn": "Inbox"}),
        )
        .await?;

    let target_title = &app.columns[app.selected_col].column.title;
    app.status_msg = format!("Moved '{}' -> {}", source.todo_title, target_title);
    app.mode = Mode::Normal;
    Ok(())
}

async fn handle_set_sub_column(app: &mut App, client: &dyn ApiClient, sub: &str) -> Result<()> {
    if let Some(card) = app.selected_card_data() {
        let card_id = card.card.id.clone();
        let title = card.todo.title.clone();
        client
            .update_card(&card_id, serde_json::json!({"subColumn": sub}))
            .await?;
        app.status_msg = format!("{}: {}", sub, title);
    }
    Ok(())
}

async fn reload_board(app: &mut App, client: &dyn ApiClient) -> Result<()> {
    let board_id = match &app.board {
        Some(b) => b.id.clone(),
        None => return Ok(()),
    };

    let saved_col = app.selected_col;
    let saved_card = app.selected_card;
    let saved_msg = app.status_msg.clone();

    let (board, columns) = App::load_board(client, &board_id).await?;
    app.board = Some(board);
    app.columns = columns;
    app.selected_col = saved_col.min(app.columns.len().saturating_sub(1));

    let count = app.visible_card_count();
    app.selected_card = if count == 0 {
        0
    } else {
        saved_card.min(count - 1)
    };
    app.status_msg = saved_msg;

    Ok(())
}
