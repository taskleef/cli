use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::{Priority, SubColumn};

use super::app::{App, ColumnRow, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::BoardList => draw_board_list(frame, app),
        Screen::Board => draw_board_screen(frame, app),
    }
}

// --- Board list screen ---

fn draw_board_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(frame.area());

    // Title
    let title = Paragraph::new(Line::from(Span::styled(
        " Boards",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(title, chunks[0]);

    // Board list
    if app.boards.is_empty() {
        let msg = Paragraph::new("  No boards found.")
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(msg, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .boards
            .iter()
            .enumerate()
            .map(|(i, board)| {
                let is_selected = i == app.selected_board;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(
                    format!("  {}  {}", &board.id[..8.min(board.id.len())], board.title),
                    style,
                )))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(list, chunks[1]);
    }

    // Status bar
    let bar = Paragraph::new(Line::from(vec![
        Span::styled(" [↑/↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" select  "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" open  "),
        Span::styled("[q]", Style::default().fg(Color::Cyan)),
        Span::raw("uit"),
    ]));
    frame.render_widget(bar, chunks[2]);
}

// --- Board screen ---

fn draw_board_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(frame.area());

    draw_title(frame, chunks[0], app);
    draw_board(frame, chunks[1], app);
    draw_status_bar(frame, chunks[2], app);

    if app.is_detail_visible() {
        draw_detail_overlay(frame, frame.area(), app);
    }
}

fn draw_title(frame: &mut Frame, area: Rect, app: &App) {
    let board_title = app
        .board
        .as_ref()
        .map(|b| b.title.as_str())
        .unwrap_or("Board");

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", board_title),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} columns", app.columns.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(title, area);
}

fn draw_board(frame: &mut Frame, area: Rect, app: &App) {
    if app.columns.is_empty() {
        let msg = Paragraph::new("No columns found.")
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(msg, area);
        return;
    }

    let constraints: Vec<Constraint> = app
        .columns
        .iter()
        .map(|_| Constraint::Ratio(1, app.columns.len() as u32))
        .collect();

    let col_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, resolved_col) in app.columns.iter().enumerate() {
        let is_selected_col = i == app.selected_col;
        let (rows, selectable) = resolved_col.build_rows();

        // Header with total active count and WIP
        let active_count = resolved_col.cards_by_sub(&SubColumn::Inbox).len();
        let header = match resolved_col.column.wip_limit {
            Some(wip) if wip > 0 => {
                let wip_style = if active_count >= wip as usize {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                Line::from(vec![
                    Span::raw(format!("{} ", resolved_col.column.title)),
                    Span::styled(format!("({}/{})", active_count, wip), wip_style),
                ])
            }
            _ => {
                let total = resolved_col.cards.len();
                Line::from(format!("{} ({})", resolved_col.column.title, total))
            }
        };

        let border_style = if is_selected_col {
            if app.is_move_mode() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            }
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(header)
            .borders(Borders::ALL)
            .border_style(border_style);

        // Build list items from rows
        let items: Vec<ListItem> = rows
            .iter()
            .enumerate()
            .map(|(row_idx, row)| match row {
                ColumnRow::SectionHeader { label, count, sub_column } => {
                    let (icon, color) = match sub_column {
                        SubColumn::Inbox => ("○", Color::White),
                        SubColumn::Blocked => ("⊗", Color::Red),
                        SubColumn::Done => ("✓", Color::Green),
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{} {} ({})", icon, label, count),
                            Style::default()
                                .fg(color)
                                .add_modifier(Modifier::DIM),
                        ),
                    ]))
                }
                ColumnRow::Card { resolved } => {
                    // Is this card currently selected?
                    let is_selected = is_selected_col
                        && !app.is_move_mode()
                        && selectable
                            .get(app.selected_card)
                            .map(|idx| *idx == row_idx)
                            .unwrap_or(false);

                    let sub = resolved
                        .card
                        .sub_column
                        .as_ref()
                        .unwrap_or(&SubColumn::Inbox);

                    let priority_icon = match &resolved.todo.priority {
                        Some(Priority::High) => Span::styled("● ", Style::default().fg(Color::Red)),
                        Some(Priority::Medium) => {
                            Span::styled("● ", Style::default().fg(Color::Yellow))
                        }
                        Some(Priority::Low) => {
                            Span::styled("● ", Style::default().fg(Color::Green))
                        }
                        None => Span::styled("○ ", Style::default().fg(Color::DarkGray)),
                    };

                    let title_style = if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        match sub {
                            SubColumn::Done => Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::DIM),
                            SubColumn::Blocked => Style::default().fg(Color::Red),
                            SubColumn::Inbox => Style::default().fg(Color::White),
                        }
                    };

                    let line = Line::from(vec![
                        Span::raw("  "), // indent under section header
                        priority_icon,
                        Span::styled(&resolved.todo.title, title_style),
                    ]);
                    ListItem::new(line)
                }
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, col_areas[i]);
    }
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let content = if !app.status_msg.is_empty() {
        Line::from(Span::styled(
            format!(" {}", app.status_msg),
            Style::default().fg(Color::Yellow),
        ))
    } else {
        Line::from(vec![
            Span::styled(" [←/→]", Style::default().fg(Color::Cyan)),
            Span::raw(" column  "),
            Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
            Span::raw(" card  "),
            Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
            Span::raw(" detail  "),
            Span::styled("[m]", Style::default().fg(Color::Cyan)),
            Span::raw("ove  "),
            Span::styled("[d]", Style::default().fg(Color::Cyan)),
            Span::raw("one  "),
            Span::styled("[b]", Style::default().fg(Color::Cyan)),
            Span::raw("locked  "),
            Span::styled("[r]", Style::default().fg(Color::Cyan)),
            Span::raw("efresh  "),
            Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
            Span::raw(" back  "),
            Span::styled("[q]", Style::default().fg(Color::Cyan)),
            Span::raw("uit"),
        ])
    };

    let bar = Paragraph::new(content);
    frame.render_widget(bar, area);
}

// --- Detail overlay ---

fn animated_rect(area: Rect, progress: f64) -> Rect {
    let target_w = (area.width as f64 * 0.6).max(40.0).min(area.width as f64);
    let target_h = (area.height as f64 * 0.7).max(10.0).min(area.height as f64);
    let t = 1.0 - (1.0 - progress).powi(3);
    let w = (target_w * t).max(2.0) as u16;
    let h = (target_h * t).max(1.0) as u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w.min(area.width), h.min(area.height))
}

fn draw_detail_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let detail = match &app.detail {
        Some(d) => d,
        None => return,
    };

    let progress = app.detail_progress();
    let panel = animated_rect(area, progress);
    frame.render_widget(Clear, panel);

    let todo = &detail.todo;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        &todo.title,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Sub-column status
    let sub = detail
        .card
        .sub_column
        .as_ref()
        .unwrap_or(&SubColumn::Inbox);
    let (sub_label, sub_color) = match sub {
        SubColumn::Inbox => ("Active", Color::White),
        SubColumn::Blocked => ("Blocked", Color::Red),
        SubColumn::Done => ("Done", Color::Green),
    };
    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
        Span::styled(sub_label, Style::default().fg(sub_color)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Column: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&detail.column_title, Style::default().fg(Color::Cyan)),
    ]));

    if let Some(ref priority) = todo.priority {
        let (label, color) = match priority {
            Priority::High => ("High", Color::Red),
            Priority::Medium => ("Medium", Color::Yellow),
            Priority::Low => ("Low", Color::Green),
        };
        lines.push(Line::from(vec![
            Span::styled("Priority: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("● {}", label), Style::default().fg(color)),
        ]));
    }

    if let Some(ref due) = todo.due_date {
        if !due.is_empty() && due != "null" {
            let date_part = due.split('T').next().unwrap_or(due);
            lines.push(Line::from(vec![
                Span::styled("Due: ", Style::default().fg(Color::DarkGray)),
                Span::raw(date_part),
            ]));
        }
    }

    if let Some(ref desc) = todo.description {
        if !desc.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Description",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            for line in desc.lines() {
                lines.push(Line::from(Span::raw(format!("  {}", line))));
            }
        }
    }

    if let Some(ref subtasks) = todo.subtasks {
        if !subtasks.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("Subtasks ({})", subtasks.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            for st in subtasks {
                lines.push(Line::from(format!("  ○ {}", st.title)));
            }
        }
    }

    if let Some(ref tags) = todo.tags {
        if !tags.is_empty() {
            lines.push(Line::from(""));
            let tag_spans: Vec<Span> = tags
                .iter()
                .flat_map(|t| {
                    vec![
                        Span::styled(
                            format!(" {} ", t.name),
                            Style::default().fg(Color::Black).bg(Color::Cyan),
                        ),
                        Span::raw(" "),
                    ]
                })
                .collect();
            lines.push(Line::from(tag_spans));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Esc to close",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Card Detail ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, panel);
}
