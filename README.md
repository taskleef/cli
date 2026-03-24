# Taskleef CLI

A command-line interface for managing todos with the [Taskleef](https://taskleef.com) todo app.

## Prerequisites

- `curl` - for making API requests
- `jq` - for parsing JSON responses

### Installing jq

**macOS:**
```bash
brew install jq
```

**Ubuntu/Debian:**
```bash
sudo apt-get install jq
```

## Installation

### Option 1: Clone and add to PATH

```bash
git clone https://github.com/Xatter/taskleef.git
cd taskleef
chmod +x todo
```

Add to your PATH by adding this to your `~/.bashrc` or `~/.zshrc`:
```bash
export PATH="$PATH:/path/to/taskleef"
```

### Option 2: Copy to a directory in your PATH

```bash
git clone https://github.com/Xatter/taskleef.git
sudo cp taskleef/todo /usr/local/bin/
```

## Configuration

### Option 1: Environment Variable

1. Go to [taskleef.com](https://taskleef.com) and generate an API key
2. Set the `TASKLEEF_API_KEY` environment variable:

```bash
export TASKLEEF_API_KEY=your-api-key-here
```

Add this to your `~/.bashrc` or `~/.zshrc` to make it permanent.

### Option 2: Auth File

Create an auth file (e.g., `~/.taskleef.auth`) containing:
```bash
TASKLEEF_API_KEY=your-api-key-here
```

Then use the `--auth-file` flag:
```bash
todo --auth-file ~/.taskleef.auth list
todo -a ~/.taskleef.auth list
```

This is useful for managing multiple accounts or keeping credentials separate.

### Optional: Custom API URL

If you're running your own Taskleef server, set:
```bash
export TASKLEEF_API_URL=https://your-server.com
```

### Optional: Command Alias

If you prefer using `tl` instead of `todo`, add this alias to your `~/.bashrc` or `~/.zshrc`:
```bash
alias tl=todo
```

## Tab Completion

### Bash
```bash
source /path/to/taskleef/todo-completion.bash
```

### Zsh
```bash
source /path/to/taskleef/todo-completion.zsh
```

Add the appropriate line to your `~/.bashrc` or `~/.zshrc` to enable completion on startup.

## MCP Server (AI Integration)

Taskleef provides a [Model Context Protocol](https://modelcontextprotocol.io/) server, allowing AI assistants like Claude Code and Claude Desktop to manage your todos, boards, and Kanban workflows directly.

### Claude Code

```bash
claude mcp add --transport http taskleef https://taskleef.com/mcp/messages -H "X-API-Key: YOUR_API_KEY"
```

Then restart Claude Code and verify with `claude mcp list`.

### Manual Configuration

Add to `~/.claude.json`:

```json
{
  "mcpServers": {
    "taskleef": {
      "type": "http",
      "url": "https://taskleef.com/mcp/messages",
      "headers": {
        "X-API-Key": "your-api-key-here"
      }
    }
  }
}
```

### Claude Desktop

Add to your MCP settings file:

```json
{
  "mcpServers": {
    "taskleef": {
      "transport": {
        "type": "sse",
        "url": "https://taskleef.com/mcp/sse",
        "headers": {
          "X-API-Key": "your-api-key-here"
        }
      }
    }
  }
}
```

See the [full API documentation](https://taskleef.com/docs) for all 39 available tools across todos, boards, columns, cards, members, tags, and comments.

## Usage

### Global Options

```bash
todo [--auth-file <path>] <command> [args]
todo [-a <path>] <command> [args]
```

### Basic Commands

```bash
# List pending todos
todo list
todo ls

# List all todos (including completed)
todo list -a

# Add a new todo
todo add "Buy groceries"

# Quick add (without 'add' keyword)
todo "Buy groceries"

# Show a todo with details and subtasks
todo show <title-or-id>

# Mark a todo as complete
todo complete <title-or-id>
todo done <title-or-id>

# Delete a todo
todo delete <title-or-id>
todo rm <title-or-id>
```

### Inbox

```bash
# List todos not assigned to any project
todo inbox
```

### Subtasks

```bash
# Add a subtask to a todo
todo subtask <parent-title-or-id> "Subtask title"
```

### Projects

```bash
# List all projects
todo project list

# Create a new project
todo project add "Project Name"

# Show project with its todos
todo project show <project-name-or-id>

# Delete a project
todo project delete <project-name-or-id>

# Add a todo to a project
todo project add-todo <project-name-or-id> <todo-title-or-id>

# Remove a todo from a project
todo project remove-todo <project-name-or-id> <todo-title-or-id>
```

### Boards (Kanban)

```bash
# Show default board (ASCII view)
todo board

# List all accessible boards
todo board list

# Show a specific board with columns and cards
todo board show <board-name-or-id>

# List cards in a specific column
todo board column <column-name-or-id>

# Move a card to a different column
todo board move <card-title-or-id> <column-name-or-id>

# Mark a card as done in its current column
todo board done <card-title-or-id>

# Assign a card to the current user
todo board assign <card-title-or-id>

# Delete all cards in a column
todo board clear <column-name-or-id>
```

## Finding Todos, Projects, and Boards

Commands that accept an identifier support:

- **ID prefix**: The first few characters of the UUID (e.g., `abc12`)
- **Title match**: Partial, case-insensitive title match (e.g., `groceries` matches "Buy groceries")

## Examples

```bash
# Add a todo
$ todo add "Review pull request"
Created: Review pull request (a1b2c)

# List todos
$ todo ls
Pending todos:

  ○ a1b2c  Review pull request
  ● d3e4f  2024-01-15  Fix login bug

# Complete a todo by title
$ todo done "pull request"
Completed: Review pull request

# Create a project and add todos
$ todo project add "Website Redesign"
Created project: Website Redesign (x7y8z)

$ todo project add-todo "Website" "Fix login"
Added todo to project: Website Redesign

# View a kanban board
$ todo board
┌─────────────┬─────────────┬─────────────┐
│ Backlog     │ In Progress │ Done        │
├─────────────┼─────────────┼─────────────┤
│ ○ Fix bug   │ ● Feature A │ ✓ Setup CI  │
│ ○ Add tests │             │             │
└─────────────┴─────────────┴─────────────┘

# Move a card to a different column
$ todo board move "Feature A" "Done"
Moved: Feature A -> Done
```

## Priority Indicators

- ○ No priority
- ● (green) Low priority
- ● (yellow) Medium priority
- ● (red) High priority

## License

MIT
// test integration Tue Mar 24 09:31:17 EDT 2026
