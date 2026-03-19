use colored::Colorize;
use futures::future::join_all;

use crate::client::ApiClient;
use crate::display::{short_id, sub_column_icon, truncate};
use crate::error::{Result, TaskleefError};
use crate::models::SubColumn;
use crate::resolve::{resolve_board, resolve_card, resolve_column};

pub async fn list(client: &dyn ApiClient) -> Result<()> {
    let boards = client.list_boards().await?;

    if boards.is_empty() {
        println!("{}", "No boards found.".yellow());
        return Ok(());
    }

    println!("{}", "Boards:".blue());
    println!();
    for b in &boards {
        println!("  {}  {}", short_id(&b.id), b.title);
    }

    Ok(())
}

pub async fn show(client: &dyn ApiClient, query: &str) -> Result<()> {
    let board_id = resolve_board(client, query).await?;
    let board = client.get_board(&board_id).await?;
    let mut columns = client.list_columns(&board_id).await?;
    columns.sort_by_key(|c| c.order.unwrap_or(0));

    println!("{}", format!("Board: {}", board.title).blue());
    println!();

    // Fetch all column cards in parallel
    let card_futures: Vec<_> = columns
        .iter()
        .map(|col| {
            let col_id = col.id.clone();
            async move {
                let cards = client.list_cards(&col_id).await?;
                Ok::<_, TaskleefError>((col_id, cards))
            }
        })
        .collect();
    let card_results = join_all(card_futures).await;

    for (i, col) in columns.iter().enumerate() {
        let cards = match &card_results[i] {
            Ok((_, cards)) => cards,
            Err(_) => continue,
        };

        let inbox_count = cards
            .iter()
            .filter(|c| c.sub_column.as_ref() == Some(&SubColumn::Inbox))
            .count();
        let done_count = cards
            .iter()
            .filter(|c| c.sub_column.as_ref() == Some(&SubColumn::Done))
            .count();
        let blocked_count = cards
            .iter()
            .filter(|c| c.sub_column.as_ref() == Some(&SubColumn::Blocked))
            .count();

        let wip_display = match col.wip_limit {
            Some(wip) if wip > 0 => {
                if inbox_count >= wip as usize {
                    format!(" {}", format!("({}/{})", inbox_count, wip).red())
                } else {
                    format!(" {}", format!("({}/{})", inbox_count, wip).dimmed())
                }
            }
            _ => String::new(),
        };

        println!("  {}{}", col.title.blue(), wip_display);
        println!(
            "    {} active, {} done, {} blocked",
            inbox_count, done_count, blocked_count
        );

        // Show first 3 card titles from Inbox
        let inbox_cards: Vec<_> = cards
            .iter()
            .filter(|c| c.sub_column.as_ref() == Some(&SubColumn::Inbox))
            .take(3)
            .collect();

        // Fetch todo details for preview cards in parallel
        let todo_futures: Vec<_> = inbox_cards
            .iter()
            .map(|card| {
                let todo_id = card.todo_id.clone();
                async move { client.get_todo(&todo_id).await }
            })
            .collect();
        let todo_results = join_all(todo_futures).await;

        for result in &todo_results {
            if let Ok(todo) = result {
                let title = truncate(&todo.title, 40);
                println!("      ○ {}", title);
            }
        }
        println!();
    }

    Ok(())
}

pub async fn column(client: &dyn ApiClient, query: &str) -> Result<()> {
    let board_id = resolve_board(client, "").await?;
    let column_id = resolve_column(client, &board_id, query).await?;

    // Get column title
    let columns = client.list_columns(&board_id).await?;
    let col_title = columns
        .iter()
        .find(|c| c.id == column_id)
        .map(|c| c.title.as_str())
        .unwrap_or("Unknown");

    let cards = client.list_cards(&column_id).await?;

    println!("{}:", col_title.blue());
    println!();

    if cards.is_empty() {
        println!("  {}", "(empty)".dimmed());
        return Ok(());
    }

    // Group by SubColumn
    for sub in &[SubColumn::Inbox, SubColumn::Blocked, SubColumn::Done] {
        let sub_cards: Vec<_> = cards
            .iter()
            .filter(|c| c.sub_column.as_ref() == Some(sub))
            .collect();

        if sub_cards.is_empty() {
            continue;
        }

        let icon = sub_column_icon(sub);
        let sub_name = match sub {
            SubColumn::Inbox => "Inbox",
            SubColumn::Blocked => "Blocked",
            SubColumn::Done => "Done",
        };
        println!("  {} {} ({}):", icon, sub_name, sub_cards.len());

        // Fetch todo details in parallel
        let todo_futures: Vec<_> = sub_cards
            .iter()
            .map(|card| {
                let todo_id = card.todo_id.clone();
                async move { client.get_todo(&todo_id).await }
            })
            .collect();
        let todo_results = join_all(todo_futures).await;

        for (j, result) in todo_results.iter().enumerate() {
            if let Ok(todo) = result {
                let sid = &sub_cards[j].todo_id[..8.min(sub_cards[j].todo_id.len())];
                println!("    {}  {}", sid.dimmed(), todo.title);
            }
        }
    }

    Ok(())
}

pub async fn move_card(client: &dyn ApiClient, card_query: &str, column_query: &str) -> Result<()> {
    let board_id = resolve_board(client, "").await?;
    let card_match = resolve_card(client, &board_id, card_query).await?;
    let target_col_id = resolve_column(client, &board_id, column_query).await?;

    client
        .update_card(
            &card_match.card_id,
            serde_json::json!({"columnId": target_col_id, "subColumn": "Inbox"}),
        )
        .await?;

    let todo = client.get_todo(&card_match.todo_id).await?;
    let columns = client.list_columns(&board_id).await?;
    let col_title = columns
        .iter()
        .find(|c| c.id == target_col_id)
        .map(|c| c.title.as_str())
        .unwrap_or("Unknown");

    println!("{} {} -> {}", "Moved:".green(), todo.title, col_title);
    Ok(())
}

pub async fn done(client: &dyn ApiClient, card_query: &str) -> Result<()> {
    let board_id = resolve_board(client, "").await?;
    let card_match = resolve_card(client, &board_id, card_query).await?;

    client
        .update_card(
            &card_match.card_id,
            serde_json::json!({"subColumn": "Done"}),
        )
        .await?;

    let todo = client.get_todo(&card_match.todo_id).await?;
    println!("{} {}", "Done:".green(), todo.title);
    Ok(())
}

pub async fn assign(client: &dyn ApiClient, card_query: &str) -> Result<()> {
    let profile = client.get_profile().await?;
    let board_id = resolve_board(client, "").await?;
    let card_match = resolve_card(client, &board_id, card_query).await?;

    let todo = client.get_todo(&card_match.todo_id).await?;

    client
        .update_todo(
            &card_match.todo_id,
            serde_json::json!({
                "title": todo.title,
                "description": todo.description,
                "assigneeUserId": profile.id
            }),
        )
        .await?;

    println!("{} {}", "Assigned:".green(), todo.title);
    Ok(())
}

pub async fn clear(client: &dyn ApiClient, column_query: &str) -> Result<()> {
    let board_id = resolve_board(client, "").await?;
    let column_id = resolve_column(client, &board_id, column_query).await?;

    let columns = client.list_columns(&board_id).await?;
    let col_title = columns
        .iter()
        .find(|c| c.id == column_id)
        .map(|c| c.title.as_str())
        .unwrap_or("Unknown");

    let cards = client.list_cards(&column_id).await?;

    if cards.is_empty() {
        println!("{}", format!("Column '{}' is already empty", col_title).yellow());
        return Ok(());
    }

    // Confirmation prompt
    println!(
        "{}",
        format!(
            "Warning: This will delete {} cards from '{}'",
            cards.len(),
            col_title
        )
        .red()
    );

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("Cancelled");
        return Ok(());
    }

    // Delete all cards in parallel
    let delete_futures: Vec<_> = cards
        .iter()
        .map(|card| {
            let card_id = card.id.clone();
            async move { client.delete_card(&card_id).await }
        })
        .collect();
    let results = join_all(delete_futures).await;
    let deleted = results.iter().filter(|r| r.is_ok()).count();

    println!(
        "{}",
        format!("Deleted {} cards from {}", deleted, col_title).green()
    );
    Ok(())
}
