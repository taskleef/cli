use colored::Colorize;

use crate::client::ApiClient;
use crate::display::{format_todo_detail, format_todo_line, short_id};
use crate::error::Result;
use crate::resolve::resolve_todo;

pub async fn add(client: &dyn ApiClient, title: &str) -> Result<()> {
    let todo = client.create_todo(title).await?;
    let sid = short_id(&todo.id);
    println!("{} {} ({})", "Created:".green(), title, sid);
    Ok(())
}

pub async fn list(client: &dyn ApiClient, show_all: bool) -> Result<()> {
    let mut todos = client.list_todos().await?;

    let pending_count = todos.iter().filter(|t| !t.is_completed.unwrap_or(false)).count();
    let completed_count = todos.iter().filter(|t| t.is_completed.unwrap_or(false)).count();

    if pending_count == 0 && !show_all {
        println!("{}", "No pending todos!".green());
        if completed_count > 0 {
            println!(
                "{}",
                format!("({} completed - use 'taskleef list -a' to show)", completed_count).dimmed()
            );
        }
        return Ok(());
    }

    if show_all {
        println!("{}", "All todos:".blue());
    } else {
        println!("{}", "Pending todos:".blue());
    }
    println!();

    // Sort: completed items last, then by due date
    todos.sort_by(|a, b| {
        let a_completed = a.is_completed.unwrap_or(false);
        let b_completed = b.is_completed.unwrap_or(false);
        a_completed.cmp(&b_completed).then_with(|| {
            let a_due = a.due_date.as_deref().unwrap_or("9999");
            let b_due = b.due_date.as_deref().unwrap_or("9999");
            a_due.cmp(b_due)
        })
    });

    for todo in &todos {
        if !show_all && todo.is_completed.unwrap_or(false) {
            continue;
        }
        println!("{}", format_todo_line(todo));
    }

    Ok(())
}

pub async fn inbox(client: &dyn ApiClient) -> Result<()> {
    let mut todos = client.list_inbox().await?;

    if todos.is_empty() {
        println!("{}", "Inbox is empty!".green());
        return Ok(());
    }

    println!("{}", "Inbox:".blue());
    println!();

    todos.sort_by(|a, b| {
        let a_due = a.due_date.as_deref().unwrap_or("9999");
        let b_due = b.due_date.as_deref().unwrap_or("9999");
        a_due.cmp(b_due)
    });

    for todo in &todos {
        println!("{}", format_todo_line(todo));
    }

    Ok(())
}

pub async fn show(client: &dyn ApiClient, query: &str) -> Result<()> {
    let full_id = resolve_todo(client, query).await?;
    let todo = client.get_todo(&full_id).await?;
    println!("{}", format_todo_detail(&todo));
    Ok(())
}

pub async fn complete(client: &dyn ApiClient, query: &str) -> Result<()> {
    let full_id = resolve_todo(client, query).await?;
    let resp = client.complete_todo(&full_id).await?;
    println!("{} {}", "Completed:".green(), resp.completed.title);

    if let Some(next) = resp.next {
        let next_due = next
            .due_date
            .as_deref()
            .map(|d| d.split('T').next().unwrap_or(d))
            .unwrap_or("-");
        println!("{} {}", "Next occurrence:".blue(), next_due);
    }

    Ok(())
}

pub async fn delete(client: &dyn ApiClient, query: &str) -> Result<()> {
    let full_id = resolve_todo(client, query).await?;
    let todo = client.delete_todo(&full_id).await?;
    println!("{} {}", "Deleted:".red(), todo.title);
    Ok(())
}

#[cfg(test)]
mod tests {
    // Command functions produce stdout output and call the client.
    // Integration-level tests would use wiremock or assert_cmd.
    // Unit tests for the underlying logic (formatting, resolving) are in their respective modules.
}
