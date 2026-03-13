# Taskleef MCP Server

Taskleef includes a [Model Context Protocol](https://modelcontextprotocol.io/) (MCP) server that allows AI assistants like Claude to manage your todos, boards, and Kanban workflows.

## Quick Start with Claude Code

1. Get your API key from Taskleef (Settings > API Keys)

2. Add the MCP server:
```bash
claude mcp add --transport http taskleef https://taskleef.com/mcp/messages -H "X-API-Key: YOUR_API_KEY"
```

3. Restart Claude Code

4. Verify it's working:
```bash
claude mcp list
```

You should see `taskleef` with a green checkmark.

## Configuration

### Claude Code (CLI)

Use the `claude mcp add` command:

```bash
claude mcp add --transport http taskleef https://taskleef.com/mcp/messages -H "X-API-Key: YOUR_API_KEY"
```

Or add directly to `~/.claude.json`:

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

Add to your Claude Desktop MCP settings file:

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

### Getting an API Key

1. Log in to Taskleef at https://taskleef.com
2. Go to Settings > API Keys
3. Create a new API key
4. Copy the key (it's only shown once)

## Available Tools

Taskleef MCP provides 39 tools across 7 domains:

### Todo Tools

#### list_todos
List todos with optional filters.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| status | string | No | "all", "active", or "completed" (default: "active") |
| includeDeferred | boolean | No | Include deferred todos (default: false) |
| limit | number | No | Maximum results (default: 50) |

#### create_todo
Create a new todo with natural language support.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| title | string | Yes | Todo title. Supports natural language dates ("tomorrow", "next week") and tags ("[work]") |
| description | string | No | Detailed description |
| dueDate | string | No | ISO 8601 date (optional if using natural language in title) |
| priority | string | No | "None", "Low", "Medium", "High" |
| timezone | string | No | IANA timezone for date parsing (e.g., "America/New_York") |

#### complete_todo
Toggle a todo's completion status.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| todoId | string | Yes | UUID of the todo |

#### search_todos
Search todos by title or description.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| query | string | Yes | Search text |
| status | string | No | "all", "active", or "completed" |
| limit | number | No | Maximum results (default: 20) |

#### update_todo
Update a todo's fields.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| todoId | string | Yes | UUID of the todo |
| title | string | No | New title |
| description | string | No | New description (null to clear) |
| dueDate | string | No | New due date (null to clear) |
| priority | string | No | New priority |

#### delete_todo
Permanently delete a todo.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| todoId | string | Yes | UUID of the todo |

#### get_inbox
Get todos in your inbox (not assigned to any board).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| includeDeferred | boolean | No | Include deferred todos (default: false) |

---

### Board Tools

#### board_list
List all boards the user has access to.

*No parameters required.*

#### board_get
Get a single board with full details including columns, members, and card counts.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |

#### board_create
Create a new board with default columns (Backlog, In Progress, Done).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| name | string | Yes | Board name |

#### board_update
Update board name.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |
| name | string | No | New board name |

#### board_delete
Delete a board and all its columns/cards. Underlying todos are preserved.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |

---

### Column Tools

#### column_list
List all columns for a board, ordered by position.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |

#### column_get
Get a column with its cards.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| columnId | string | Yes | UUID of the column |

#### column_create
Add a new column to a board.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |
| name | string | Yes | Column name |
| wipLimit | number | No | Work-in-progress limit |

#### column_update
Update column name or WIP limit.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| columnId | string | Yes | UUID of the column |
| name | string | No | New column name |
| wipLimit | number | No | New WIP limit (null to remove) |

#### column_reorder
Change column position on the board.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| columnId | string | Yes | UUID of the column |
| newPosition | number | Yes | New position (0-indexed) |

#### column_delete
Delete a column. Cards in column are deleted but underlying todos are preserved.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| columnId | string | Yes | UUID of the column |

---

### Card Tools

Cards represent todos on a board. A todo can exist on multiple boards as different cards.

#### card_list
List cards on a board.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |
| columnId | string | No | Filter by column |

#### card_create
Add a todo to a board as a card.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |
| todoId | string | Yes | UUID of the todo to add |
| columnId | string | No | Target column (defaults to first column) |
| subColumn | string | No | "Inbox", "Done", or "Blocked" |

#### card_move
Move a card to a different column or sub-column.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| cardId | string | Yes | UUID of the card |
| columnId | string | No | Target column |
| subColumn | string | No | "Inbox", "Done", or "Blocked" |
| position | number | No | Position in column (defaults to end) |

#### card_reorder
Change card position within its current column.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| cardId | string | Yes | UUID of the card |
| newPosition | number | Yes | New position (0-indexed) |

#### card_delete
Remove a card from a board. The underlying todo is preserved.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| cardId | string | Yes | UUID of the card |

---

### Member Tools

#### member_list
List all members of a board.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |

#### member_add
Add a user to a board by email.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |
| email | string | Yes | Email of the user to add |
| role | string | No | "Editor" or "Viewer" (default: "Editor") |

#### member_update_role
Change a member's role.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |
| userId | string | Yes | UUID of the user |
| role | string | Yes | "Editor" or "Viewer" |

#### member_remove
Remove a member from a board.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |
| userId | string | Yes | UUID of the user to remove |

#### member_leave
Current user leaves a board.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| boardId | string | Yes | UUID of the board |

---

### Tag Tools

Tags are user-scoped and can be applied to any todo.

#### tag_list
List all tags for the current user.

*No parameters required.*

#### tag_get
Get a tag with its associated todos.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| tagId | string | Yes | UUID of the tag |

#### tag_create
Create a new tag.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| name | string | Yes | Tag name |
| color | string | No | Hex color code (e.g., "#FF5733") |

#### tag_update
Update tag name or color.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| tagId | string | Yes | UUID of the tag |
| name | string | No | New tag name |
| color | string | No | New hex color code |

#### tag_delete
Delete a tag. Removes from all todos but doesn't delete the todos.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| tagId | string | Yes | UUID of the tag |

#### tag_add_to_todo
Add a tag to a todo.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| todoId | string | Yes | UUID of the todo |
| tagId | string | Yes | UUID of the tag |

#### tag_remove_from_todo
Remove a tag from a todo.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| todoId | string | Yes | UUID of the todo |
| tagId | string | Yes | UUID of the tag |

---

### Comment Tools

#### comment_list
List comments on a todo.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| todoId | string | Yes | UUID of the todo |
| limit | number | No | Maximum results (default: 50) |

#### comment_get
Get a single comment.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| commentId | string | Yes | UUID of the comment |

#### comment_create
Add a comment to a todo.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| todoId | string | Yes | UUID of the todo |
| content | string | Yes | Comment content |

#### comment_update
Edit a comment.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| commentId | string | Yes | UUID of the comment |
| content | string | Yes | New comment content |

#### comment_delete
Delete a comment.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| commentId | string | Yes | UUID of the comment |

---

## Protocol Details

- **Transport**: HTTP (recommended) or HTTP + Server-Sent Events (SSE)
- **Authentication**: API Key via `X-API-Key` header
- **HTTP Endpoint**: `POST /mcp/messages`
- **SSE Endpoint**: `GET /mcp/sse` (for SSE transport)
- **Protocol Version**: 2024-11-05

## Example Usage

Once configured, you can ask Claude to manage your tasks naturally:

- "Show me my todos for this week"
- "Create a todo to review the quarterly report by Friday [work]"
- "Move the authentication task to In Progress on the Sprint board"
- "Add a comment to the API refactor task saying I've started the review"
- "Tag all my meeting todos with the 'meetings' tag"
- "Create a new board called 'Q2 Planning'"

## Troubleshooting

### "Failed to connect" in `claude mcp list`

1. Verify your API key is correct
2. Check that the URL is accessible
3. Try restarting Claude Code

### Authentication errors

- API keys are shown only once when created. If lost, create a new one.
- Ensure the header format is correct: `-H "X-API-Key: YOUR_KEY"`

### Tools not appearing

After adding or modifying MCP configuration, restart Claude Code for changes to take effect.
