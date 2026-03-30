use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(name = "taskleef", about = "CLI for taskleef.com")]
pub struct Cli {
    /// Load API key from auth file
    #[arg(long = "auth-file", global = true)]
    pub auth_file: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,

    /// When no subcommand is recognized, treat remaining args as a quick-add title
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub rest: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new todo
    Add {
        /// Todo title
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        title: Vec<String>,
    },

    /// List pending todos
    #[command(alias = "ls")]
    List {
        /// Include completed todos
        #[arg(short = 'a', long = "all")]
        all: bool,
    },

    /// List todos in inbox (not in any project)
    Inbox,

    /// Show a todo with its subtasks
    Show {
        /// Todo title (partial match) or ID prefix
        query: String,
    },

    /// Mark a todo as complete
    #[command(alias = "done")]
    Complete {
        /// Todo title (partial match) or ID prefix
        query: String,
    },

    /// Delete a todo
    #[command(alias = "rm")]
    Delete {
        /// Todo title (partial match) or ID prefix
        query: String,
    },

    /// Add a subtask to a todo
    Subtask {
        /// Parent todo title or ID
        parent: String,
        /// Subtask title
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        title: Vec<String>,
    },

    /// Project management commands
    Project {
        #[command(subcommand)]
        command: Option<ProjectCommand>,
    },

    /// Board management commands
    Board {
        #[command(subcommand)]
        command: Option<BoardCommand>,
    },

    /// Interactive terminal UI
    #[command(alias = "t")]
    Tui,

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    /// List all projects
    #[command(alias = "ls")]
    List,

    /// Create a new project
    Add {
        /// Project title
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        title: Vec<String>,
    },

    /// Show project with its todos
    Show {
        /// Project name or ID
        query: String,
    },

    /// Delete a project
    #[command(alias = "rm")]
    Delete {
        /// Project name or ID
        query: String,
    },

    /// Add a todo to a project
    AddTodo {
        /// Project name or ID
        project: String,
        /// Todo title or ID
        todo: String,
    },

    /// Remove a todo from a project
    RemoveTodo {
        /// Project name or ID
        project: String,
        /// Todo title or ID
        todo: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum BoardCommand {
    /// List all accessible boards
    #[command(alias = "ls")]
    List,

    /// Show board with columns and cards
    Show {
        /// Board name or ID (optional, defaults to first board)
        query: Option<String>,
    },

    /// List cards in a column (grouped by sub-column)
    #[command(alias = "col")]
    Column {
        /// Column name or ID
        query: String,
    },

    /// Move card to a column
    #[command(alias = "mv")]
    Move {
        /// Card's todo title or ID
        card: String,
        /// Target column name or ID
        column: String,
    },

    /// Mark card done in current column
    Done {
        /// Card's todo title or ID
        card: String,
    },

    /// Assign card to current user
    Assign {
        /// Card's todo title or ID
        card: String,
    },

    /// Delete all cards in a column
    Clear {
        /// Column name or ID
        column: String,
    },
}

/// Parse CLI arguments with implicit add fallback.
/// If clap can't parse a known subcommand, treat args as a quick-add title.
pub fn parse_args() -> Cli {
    match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // If it's a missing subcommand / unrecognized, try implicit add
            if e.kind() == clap::error::ErrorKind::InvalidSubcommand
                || e.kind() == clap::error::ErrorKind::UnknownArgument
            {
                // Get raw args, skip program name
                let args: Vec<String> = std::env::args().skip(1).collect();
                // Filter out --auth-file and its value
                let mut title_parts = Vec::new();
                let mut skip_next = false;
                let mut auth_file = None;
                for (i, arg) in args.iter().enumerate() {
                    if skip_next {
                        skip_next = false;
                        continue;
                    }
                    if arg == "--auth-file" {
                        auth_file = args.get(i + 1).cloned();
                        skip_next = true;
                        continue;
                    }
                    title_parts.push(arg.clone());
                }

                if title_parts.is_empty() {
                    e.exit();
                }

                Cli {
                    auth_file,
                    command: Some(Command::Add {
                        title: title_parts,
                    }),
                    rest: vec![],
                }
            } else {
                e.exit()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_add() {
        let cli = Cli::try_parse_from(["taskleef", "add", "Buy milk"]).unwrap();
        match cli.command {
            Some(Command::Add { title }) => assert_eq!(title, vec!["Buy milk"]),
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_multi_word() {
        let cli = Cli::try_parse_from(["taskleef", "add", "Buy", "milk"]).unwrap();
        match cli.command {
            Some(Command::Add { title }) => assert_eq!(title, vec!["Buy", "milk"]),
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_list() {
        let cli = Cli::try_parse_from(["taskleef", "list"]).unwrap();
        match cli.command {
            Some(Command::List { all }) => assert!(!all),
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_list_all() {
        let cli = Cli::try_parse_from(["taskleef", "list", "-a"]).unwrap();
        match cli.command {
            Some(Command::List { all }) => assert!(all),
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_ls_alias() {
        let cli = Cli::try_parse_from(["taskleef", "ls"]).unwrap();
        assert!(matches!(cli.command, Some(Command::List { .. })));
    }

    #[test]
    fn test_parse_complete() {
        let cli = Cli::try_parse_from(["taskleef", "complete", "milk"]).unwrap();
        match cli.command {
            Some(Command::Complete { query }) => assert_eq!(query, "milk"),
            _ => panic!("Expected Complete command"),
        }
    }

    #[test]
    fn test_parse_done_alias() {
        let cli = Cli::try_parse_from(["taskleef", "done", "milk"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Complete { .. })));
    }

    #[test]
    fn test_parse_delete() {
        let cli = Cli::try_parse_from(["taskleef", "delete", "milk"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Delete { .. })));
    }

    #[test]
    fn test_parse_rm_alias() {
        let cli = Cli::try_parse_from(["taskleef", "rm", "milk"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Delete { .. })));
    }

    #[test]
    fn test_parse_show() {
        let cli = Cli::try_parse_from(["taskleef", "show", "milk"]).unwrap();
        match cli.command {
            Some(Command::Show { query }) => assert_eq!(query, "milk"),
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_parse_inbox() {
        let cli = Cli::try_parse_from(["taskleef", "inbox"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Inbox)));
    }

    #[test]
    fn test_parse_subtask() {
        let cli = Cli::try_parse_from(["taskleef", "subtask", "parent", "child", "task"]).unwrap();
        match cli.command {
            Some(Command::Subtask { parent, title }) => {
                assert_eq!(parent, "parent");
                assert_eq!(title, vec!["child", "task"]);
            }
            _ => panic!("Expected Subtask command"),
        }
    }

    #[test]
    fn test_parse_board_list() {
        let cli = Cli::try_parse_from(["taskleef", "board", "list"]).unwrap();
        match cli.command {
            Some(Command::Board {
                command: Some(BoardCommand::List),
            }) => {}
            _ => panic!("Expected Board List"),
        }
    }

    #[test]
    fn test_parse_board_show() {
        let cli = Cli::try_parse_from(["taskleef", "board", "show"]).unwrap();
        match cli.command {
            Some(Command::Board {
                command: Some(BoardCommand::Show { query }),
            }) => assert!(query.is_none()),
            _ => panic!("Expected Board Show"),
        }
    }

    #[test]
    fn test_parse_board_move() {
        let cli = Cli::try_parse_from(["taskleef", "board", "move", "card1", "col1"]).unwrap();
        match cli.command {
            Some(Command::Board {
                command: Some(BoardCommand::Move { card, column }),
            }) => {
                assert_eq!(card, "card1");
                assert_eq!(column, "col1");
            }
            _ => panic!("Expected Board Move"),
        }
    }

    #[test]
    fn test_parse_board_mv_alias() {
        let cli = Cli::try_parse_from(["taskleef", "board", "mv", "card1", "col1"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Board {
                command: Some(BoardCommand::Move { .. })
            })
        ));
    }

    #[test]
    fn test_parse_board_col_alias() {
        let cli = Cli::try_parse_from(["taskleef", "board", "col", "mycol"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Board {
                command: Some(BoardCommand::Column { .. })
            })
        ));
    }

    #[test]
    fn test_parse_project_list() {
        let cli = Cli::try_parse_from(["taskleef", "project", "list"]).unwrap();
        match cli.command {
            Some(Command::Project {
                command: Some(ProjectCommand::List),
            }) => {}
            _ => panic!("Expected Project List"),
        }
    }

    #[test]
    fn test_parse_project_add_todo() {
        let cli =
            Cli::try_parse_from(["taskleef", "project", "add-todo", "proj1", "todo1"]).unwrap();
        match cli.command {
            Some(Command::Project {
                command: Some(ProjectCommand::AddTodo { project, todo }),
            }) => {
                assert_eq!(project, "proj1");
                assert_eq!(todo, "todo1");
            }
            _ => panic!("Expected Project AddTodo"),
        }
    }

    #[test]
    fn test_parse_auth_file() {
        let cli =
            Cli::try_parse_from(["taskleef", "--auth-file", "/tmp/auth", "list"]).unwrap();
        assert_eq!(cli.auth_file.as_deref(), Some("/tmp/auth"));
        assert!(matches!(cli.command, Some(Command::List { .. })));
    }

    #[test]
    fn test_parse_no_command_returns_none() {
        let cli = Cli::try_parse_from(["taskleef"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_parse_completions() {
        let cli = Cli::try_parse_from(["taskleef", "completions", "bash"]).unwrap();
        match cli.command {
            Some(Command::Completions { shell }) => assert_eq!(shell, Shell::Bash),
            _ => panic!("Expected Completions command"),
        }
    }

    #[test]
    fn test_parse_tui() {
        let cli = Cli::try_parse_from(["taskleef", "tui"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Tui)));
    }

    #[test]
    fn test_parse_tui_alias() {
        let cli = Cli::try_parse_from(["taskleef", "t"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Tui)));
    }
}
