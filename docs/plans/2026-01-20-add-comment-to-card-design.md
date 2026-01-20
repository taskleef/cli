# Add Comment to Card

## Overview

Add the ability to add comments to todos/cards via the CLI and display them when viewing a todo.

## Command Interface

### Adding a comment

```bash
todo comment <todo> "This is my comment"
```

- `<todo>` accepts todo title (partial match) or ID prefix
- Uses existing `find_todo_id` helper
- Output: `Added comment: <todo title>`

### Viewing comments

Comments appear in `todo show <todo>` output after tags:

```
Buy groceries
   Due: 2026-01-20

   Subtasks:
   ○ abc12  Get milk

Tags: [shopping]

Comments:
   Jan 18 alice  Check if organic is available
   Jan 19 bob    Found a good deal at Costco
```

## API Integration

### Endpoints

- `POST /api/todos/{todoId}/comments` - Create comment
  - Body: `{"content": "comment text"}`
  - Returns: `CommentResponse`

- `GET /api/todos/{todoId}/comments` - Get comments (newest first)
  - Returns: Array of `CommentResponse`

### CommentResponse fields

- `id` - Comment UUID
- `todoId` - Parent todo UUID
- `authorId` - Author user UUID
- `authorName` - Author display name
- `content` - Comment text
- `createdAt` - ISO timestamp
- `updatedAt` - ISO timestamp (if edited)

## Implementation

### New function: `add_comment`

```bash
function add_comment() {
    local todo_query="$1"
    shift
    local content="$*"

    # Validate inputs
    # Find todo ID using find_todo_id
    # POST to /api/todos/{todoId}/comments
    # Display success message
}
```

### Modified function: `show_todo`

After displaying tags section:
1. GET `/api/todos/{todoId}/comments`
2. If comments exist, display "Comments:" section
3. Format each comment as: `   <date> <author>  <content>`

### Command routing

Add to main case statement:
```bash
comment)
    shift
    add_comment "$@"
    ;;
```

### Usage help

Add to `usage()` function:
```
Comment Commands:
  comment <todo> <message>     Add a comment to a todo
```

## Testing

Manual testing:
1. `todo comment "groceries" "Remember to check expiration dates"`
2. `todo show groceries` - verify comment appears
3. Add multiple comments, verify newest-first order
4. Test with todo ID prefix instead of title
