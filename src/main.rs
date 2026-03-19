mod cli;
mod client;
mod commands;
mod config;
mod display;
mod error;
mod models;
mod resolve;

use std::io;
use std::process;

use clap::CommandFactory;
use clap_complete::generate;
use colored::Colorize;

use cli::{BoardCommand, Cli, Command, ProjectCommand};
use client::HttpApiClient;
use config::Config;
use error::TaskleefError;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        match &e {
            TaskleefError::MissingApiKey => {
                let api_url = std::env::var("TASKLEEF_API_URL")
                    .unwrap_or_else(|_| "https://taskleef.com".to_string());
                eprintln!("{}", "Error: TASKLEEF_API_KEY environment variable not set".red());
                eprintln!("Generate an API key at {} and set:", api_url);
                eprintln!("  export TASKLEEF_API_KEY=your-api-key");
                eprintln!();
                eprintln!("Or use --auth-file to specify an auth file:");
                eprintln!("  taskleef --auth-file ~/.taskleef.auth <command>");
            }
            _ => eprintln!("{}", format!("Error: {}", e).red()),
        }
        process::exit(1);
    }
}

async fn run() -> error::Result<()> {
    let cli = cli::parse_args();
    let config = Config::load(cli.auth_file.as_deref())?;
    let client = HttpApiClient::new(&config.api_url, &config.api_key);

    match cli.command {
        Some(Command::Add { title }) => {
            let title = title.join(" ");
            if title.is_empty() {
                return Err(TaskleefError::Usage(
                    "Title is required. Usage: taskleef add \"Buy milk\"".into(),
                ));
            }
            commands::todo::add(&client, &title).await
        }

        Some(Command::List { all }) => commands::todo::list(&client, all).await,

        Some(Command::Inbox) => commands::todo::inbox(&client).await,

        Some(Command::Show { query }) => commands::todo::show(&client, &query).await,

        Some(Command::Complete { query }) => commands::todo::complete(&client, &query).await,

        Some(Command::Delete { query }) => commands::todo::delete(&client, &query).await,

        Some(Command::Subtask { parent, title }) => {
            let title = title.join(" ");
            if title.is_empty() {
                return Err(TaskleefError::Usage(
                    "Subtask title is required. Usage: taskleef subtask <parent> \"title\"".into(),
                ));
            }
            commands::subtask::add(&client, &parent, &title).await
        }

        Some(Command::Project { command }) => match command {
            Some(ProjectCommand::List) | None => commands::project::list(&client).await,
            Some(ProjectCommand::Add { title }) => {
                let title = title.join(" ");
                if title.is_empty() {
                    return Err(TaskleefError::Usage("Title is required".into()));
                }
                commands::project::add(&client, &title).await
            }
            Some(ProjectCommand::Show { query }) => commands::project::show(&client, &query).await,
            Some(ProjectCommand::Delete { query }) => {
                commands::project::delete(&client, &query).await
            }
            Some(ProjectCommand::AddTodo { project, todo }) => {
                commands::project::add_todo(&client, &project, &todo).await
            }
            Some(ProjectCommand::RemoveTodo { project, todo }) => {
                commands::project::remove_todo(&client, &project, &todo).await
            }
        },

        Some(Command::Board { command }) => match command {
            Some(BoardCommand::List) => commands::board::list(&client).await,
            Some(BoardCommand::Show { query }) => {
                commands::board::show(&client, query.as_deref().unwrap_or("")).await
            }
            Some(BoardCommand::Column { query }) => {
                commands::board::column(&client, &query).await
            }
            Some(BoardCommand::Move { card, column }) => {
                commands::board::move_card(&client, &card, &column).await
            }
            Some(BoardCommand::Done { card }) => commands::board::done(&client, &card).await,
            Some(BoardCommand::Assign { card }) => commands::board::assign(&client, &card).await,
            Some(BoardCommand::Clear { column }) => commands::board::clear(&client, &column).await,
            None => commands::board::show(&client, "").await,
        },

        Some(Command::Completions { shell }) => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "taskleef", &mut io::stdout());
            Ok(())
        }

        None => {
            // No command: default to list (like bash version)
            commands::todo::list(&client, false).await
        }
    }
}
