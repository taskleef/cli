use colored::Colorize;

use crate::client::ApiClient;
use crate::display::{format_project_line, short_id};
use crate::error::Result;
use crate::resolve::{resolve_project, resolve_todo};

pub async fn list(client: &dyn ApiClient) -> Result<()> {
    let projects = client.list_projects().await?;

    if projects.is_empty() {
        println!("{}", "No projects yet.".yellow());
        println!("Create one with: taskleef project add \"Project Name\"");
        return Ok(());
    }

    println!("{}", "Projects:".blue());
    println!();
    for p in &projects {
        println!("{}", format_project_line(&p.id, &p.title, &p.description));
    }

    Ok(())
}

pub async fn add(client: &dyn ApiClient, title: &str) -> Result<()> {
    let project = client.create_project(title).await?;
    let sid = short_id(&project.id);
    println!("{} {} ({})", "Created project:".green(), title, sid);
    Ok(())
}

pub async fn show(client: &dyn ApiClient, query: &str) -> Result<()> {
    let full_id = resolve_project(client, query).await?;
    let project = client.get_project(&full_id).await?;

    println!("{}", format!("📁 {}", project.title).blue());
    if let Some(ref desc) = project.description {
        if !desc.is_empty() {
            println!("   {}", desc);
        }
    }
    println!();

    match &project.todos {
        Some(todos) if !todos.is_empty() => {
            println!("   {}", "Todos:".green());
            for t in todos {
                println!("   ○ {}  {}", short_id(&t.id), t.title);
            }
        }
        _ => {
            println!("   {}", "No todos in this project".yellow());
        }
    }

    Ok(())
}

pub async fn delete(client: &dyn ApiClient, query: &str) -> Result<()> {
    let full_id = resolve_project(client, query).await?;
    let project = client.delete_project(&full_id).await?;
    println!("{} {}", "Deleted project:".red(), project.title);
    Ok(())
}

pub async fn add_todo(client: &dyn ApiClient, project_query: &str, todo_query: &str) -> Result<()> {
    let project_id = resolve_project(client, project_query).await?;
    let todo_id = resolve_todo(client, todo_query).await?;
    let project = client.add_todo_to_project(&project_id, &todo_id).await?;
    println!("{} {}", "Added todo to project:".green(), project.title);
    Ok(())
}

pub async fn remove_todo(client: &dyn ApiClient, project_query: &str, todo_query: &str) -> Result<()> {
    let project_id = resolve_project(client, project_query).await?;
    let todo_id = resolve_todo(client, todo_query).await?;
    let project = client
        .remove_todo_from_project(&project_id, &todo_id)
        .await?;
    println!("{} {}", "Removed todo from project:".yellow(), project.title);
    Ok(())
}
