use futures::future::join_all;

use crate::client::ApiClient;
use crate::error::{Result, TaskleefError};

/// Resolve a todo by ID prefix or case-insensitive title substring.
pub async fn resolve_todo(client: &dyn ApiClient, query: &str) -> Result<String> {
    let todos = client.list_todos().await?;

    // ID prefix match first
    for t in &todos {
        if t.id.starts_with(query) {
            return Ok(t.id.clone());
        }
    }

    // Case-insensitive title substring
    let query_lower = query.to_lowercase();
    for t in &todos {
        if t.title.to_lowercase().contains(&query_lower) {
            return Ok(t.id.clone());
        }
    }

    Err(TaskleefError::NotFound {
        entity: "todo".to_string(),
        query: query.to_string(),
    })
}

/// Resolve a project by ID prefix or case-insensitive title substring.
pub async fn resolve_project(client: &dyn ApiClient, query: &str) -> Result<String> {
    let projects = client.list_projects().await?;

    for p in &projects {
        if p.id.starts_with(query) {
            return Ok(p.id.clone());
        }
    }

    let query_lower = query.to_lowercase();
    for p in &projects {
        if p.title.to_lowercase().contains(&query_lower) {
            return Ok(p.id.clone());
        }
    }

    Err(TaskleefError::NotFound {
        entity: "project".to_string(),
        query: query.to_string(),
    })
}

/// Resolve a board by ID prefix or case-insensitive title substring.
/// Empty query returns the first board.
pub async fn resolve_board(client: &dyn ApiClient, query: &str) -> Result<String> {
    let boards = client.list_boards().await?;

    if query.is_empty() {
        return boards
            .first()
            .map(|b| b.id.clone())
            .ok_or(TaskleefError::NoBoards);
    }

    for b in &boards {
        if b.id.starts_with(query) {
            return Ok(b.id.clone());
        }
    }

    let query_lower = query.to_lowercase();
    for b in &boards {
        if b.title.to_lowercase().contains(&query_lower) {
            return Ok(b.id.clone());
        }
    }

    Err(TaskleefError::NotFound {
        entity: "board".to_string(),
        query: query.to_string(),
    })
}

/// Resolve a column within a board by ID prefix or case-insensitive title substring.
pub async fn resolve_column(client: &dyn ApiClient, board_id: &str, query: &str) -> Result<String> {
    let columns = client.list_columns(board_id).await?;

    for c in &columns {
        if c.id.starts_with(query) {
            return Ok(c.id.clone());
        }
    }

    let query_lower = query.to_lowercase();
    for c in &columns {
        if c.title.to_lowercase().contains(&query_lower) {
            return Ok(c.id.clone());
        }
    }

    Err(TaskleefError::NotFound {
        entity: "column".to_string(),
        query: query.to_string(),
    })
}

/// Result of resolving a card on a board.
pub struct CardMatch {
    pub card_id: String,
    pub todo_id: String,
    pub column_id: String,
}

/// Resolve a card by searching across all columns of a board.
/// Matches by todo ID prefix or case-insensitive title substring.
/// Fetches column cards in parallel.
pub async fn resolve_card(client: &dyn ApiClient, board_id: &str, query: &str) -> Result<CardMatch> {
    let columns = client.list_columns(board_id).await?;

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

    let results = join_all(card_futures).await;

    // First pass: ID prefix match on todo_id
    for result in &results {
        if let Ok((col_id, cards)) = result {
            for card in cards {
                if card.todo_id.starts_with(query) {
                    return Ok(CardMatch {
                        card_id: card.id.clone(),
                        todo_id: card.todo_id.clone(),
                        column_id: col_id.clone(),
                    });
                }
            }
        }
    }

    // Second pass: title match (requires fetching todo details)
    let query_lower = query.to_lowercase();
    for result in &results {
        if let Ok((col_id, cards)) = result {
            for card in cards {
                if let Ok(todo) = client.get_todo(&card.todo_id).await {
                    if todo.title.to_lowercase().contains(&query_lower) {
                        return Ok(CardMatch {
                            card_id: card.id.clone(),
                            todo_id: card.todo_id.clone(),
                            column_id: col_id.clone(),
                        });
                    }
                }
            }
        }
    }

    Err(TaskleefError::NotFound {
        entity: "card".to_string(),
        query: query.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use async_trait::async_trait;
    use serde_json::Value;
    use std::sync::Mutex;

    struct MockClient {
        todos: Vec<TodoResponse>,
        projects: Vec<ProjectResponse>,
        boards: Vec<BoardResponse>,
        columns: Vec<ColumnResponse>,
        cards: Mutex<Vec<(String, Vec<CardResponse>)>>, // (column_id, cards)
    }

    impl MockClient {
        fn new() -> Self {
            Self {
                todos: vec![],
                projects: vec![],
                boards: vec![],
                columns: vec![],
                cards: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl ApiClient for MockClient {
        async fn list_todos(&self) -> Result<Vec<TodoResponse>> {
            Ok(self.todos.clone())
        }
        async fn get_todo(&self, id: &str) -> Result<TodoResponse> {
            self.todos
                .iter()
                .find(|t| t.id == id)
                .cloned()
                .ok_or_else(|| TaskleefError::NotFound {
                    entity: "todo".into(),
                    query: id.into(),
                })
        }
        async fn create_todo(&self, _title: &str) -> Result<TodoResponse> {
            unimplemented!()
        }
        async fn complete_todo(&self, _id: &str) -> Result<CompletionResponse> {
            unimplemented!()
        }
        async fn delete_todo(&self, _id: &str) -> Result<TodoResponse> {
            unimplemented!()
        }
        async fn update_todo(&self, _id: &str, _body: Value) -> Result<TodoResponse> {
            unimplemented!()
        }
        async fn list_inbox(&self) -> Result<Vec<TodoResponse>> {
            unimplemented!()
        }
        async fn create_subtask(&self, _parent_id: &str, _title: &str) -> Result<TodoResponse> {
            unimplemented!()
        }
        async fn list_projects(&self) -> Result<Vec<ProjectResponse>> {
            Ok(self.projects.clone())
        }
        async fn get_project(&self, _id: &str) -> Result<ProjectResponse> {
            unimplemented!()
        }
        async fn create_project(&self, _title: &str) -> Result<ProjectResponse> {
            unimplemented!()
        }
        async fn delete_project(&self, _id: &str) -> Result<ProjectResponse> {
            unimplemented!()
        }
        async fn add_todo_to_project(&self, _pid: &str, _tid: &str) -> Result<ProjectResponse> {
            unimplemented!()
        }
        async fn remove_todo_from_project(&self, _pid: &str, _tid: &str) -> Result<ProjectResponse> {
            unimplemented!()
        }
        async fn list_boards(&self) -> Result<Vec<BoardResponse>> {
            Ok(self.boards.clone())
        }
        async fn get_board(&self, _id: &str) -> Result<BoardResponse> {
            unimplemented!()
        }
        async fn list_columns(&self, _board_id: &str) -> Result<Vec<ColumnResponse>> {
            Ok(self.columns.clone())
        }
        async fn list_cards(&self, column_id: &str) -> Result<Vec<CardResponse>> {
            let cards = self.cards.lock().unwrap();
            Ok(cards
                .iter()
                .find(|(cid, _)| cid == column_id)
                .map(|(_, c)| c.clone())
                .unwrap_or_default())
        }
        async fn update_card(&self, _id: &str, _body: Value) -> Result<CardResponse> {
            unimplemented!()
        }
        async fn delete_card(&self, _id: &str) -> Result<()> {
            unimplemented!()
        }
        async fn get_profile(&self) -> Result<ProfileResponse> {
            unimplemented!()
        }
    }

    fn make_todo(id: &str, title: &str) -> TodoResponse {
        TodoResponse {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            priority: None,
            due_date: None,
            is_completed: None,
            subtasks: None,
            tags: None,
            assignee_user_id: None,
        }
    }

    #[tokio::test]
    async fn test_resolve_todo_by_id_prefix() {
        let client = MockClient {
            todos: vec![make_todo("abc12345-full-id", "Buy milk")],
            ..MockClient::new()
        };
        let id = resolve_todo(&client, "abc12345").await.unwrap();
        assert_eq!(id, "abc12345-full-id");
    }

    #[tokio::test]
    async fn test_resolve_todo_by_title() {
        let client = MockClient {
            todos: vec![make_todo("abc12345-full-id", "Buy milk")],
            ..MockClient::new()
        };
        let id = resolve_todo(&client, "milk").await.unwrap();
        assert_eq!(id, "abc12345-full-id");
    }

    #[tokio::test]
    async fn test_resolve_todo_case_insensitive() {
        let client = MockClient {
            todos: vec![make_todo("abc12345-full-id", "Buy Milk")],
            ..MockClient::new()
        };
        let id = resolve_todo(&client, "buy milk").await.unwrap();
        assert_eq!(id, "abc12345-full-id");
    }

    #[tokio::test]
    async fn test_resolve_todo_not_found() {
        let client = MockClient {
            todos: vec![make_todo("abc12345-full-id", "Buy milk")],
            ..MockClient::new()
        };
        let result = resolve_todo(&client, "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resolve_todo_id_takes_precedence() {
        let client = MockClient {
            todos: vec![
                make_todo("abc12345-full-id", "Something else"),
                make_todo("def67890-full-id", "abc12345 is in the title"),
            ],
            ..MockClient::new()
        };
        // Should match by ID prefix, not title
        let id = resolve_todo(&client, "abc12345").await.unwrap();
        assert_eq!(id, "abc12345-full-id");
    }

    #[tokio::test]
    async fn test_resolve_project_by_title() {
        let client = MockClient {
            projects: vec![ProjectResponse {
                id: "p1-full".to_string(),
                title: "My Project".to_string(),
                description: None,
                todos: None,
            }],
            ..MockClient::new()
        };
        let id = resolve_project(&client, "project").await.unwrap();
        assert_eq!(id, "p1-full");
    }

    #[tokio::test]
    async fn test_resolve_board_empty_returns_first() {
        let client = MockClient {
            boards: vec![
                BoardResponse {
                    id: "b1".to_string(),
                    title: "First Board".to_string(),
                },
                BoardResponse {
                    id: "b2".to_string(),
                    title: "Second Board".to_string(),
                },
            ],
            ..MockClient::new()
        };
        let id = resolve_board(&client, "").await.unwrap();
        assert_eq!(id, "b1");
    }

    #[tokio::test]
    async fn test_resolve_board_empty_no_boards() {
        let client = MockClient::new();
        let result = resolve_board(&client, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resolve_column() {
        let client = MockClient {
            columns: vec![
                ColumnResponse {
                    id: "col1".to_string(),
                    title: "To Do".to_string(),
                    order: Some(0),
                    wip_limit: None,
                },
                ColumnResponse {
                    id: "col2".to_string(),
                    title: "In Progress".to_string(),
                    order: Some(1),
                    wip_limit: None,
                },
            ],
            ..MockClient::new()
        };
        let id = resolve_column(&client, "board1", "progress").await.unwrap();
        assert_eq!(id, "col2");
    }

    #[tokio::test]
    async fn test_resolve_card_by_todo_id() {
        let client = MockClient {
            columns: vec![ColumnResponse {
                id: "col1".to_string(),
                title: "To Do".to_string(),
                order: Some(0),
                wip_limit: None,
            }],
            todos: vec![make_todo("todo1-full", "Task A")],
            cards: Mutex::new(vec![(
                "col1".to_string(),
                vec![CardResponse {
                    id: "card1".to_string(),
                    todo_id: "todo1-full".to_string(),
                    sub_column: Some(SubColumn::Inbox),
                }],
            )]),
            ..MockClient::new()
        };
        let m = resolve_card(&client, "board1", "todo1").await.unwrap();
        assert_eq!(m.card_id, "card1");
        assert_eq!(m.todo_id, "todo1-full");
        assert_eq!(m.column_id, "col1");
    }

    #[tokio::test]
    async fn test_resolve_card_by_title() {
        let client = MockClient {
            columns: vec![ColumnResponse {
                id: "col1".to_string(),
                title: "To Do".to_string(),
                order: Some(0),
                wip_limit: None,
            }],
            todos: vec![make_todo("todo1-full", "My Special Task")],
            cards: Mutex::new(vec![(
                "col1".to_string(),
                vec![CardResponse {
                    id: "card1".to_string(),
                    todo_id: "todo1-full".to_string(),
                    sub_column: Some(SubColumn::Inbox),
                }],
            )]),
            ..MockClient::new()
        };
        let m = resolve_card(&client, "board1", "special").await.unwrap();
        assert_eq!(m.card_id, "card1");
    }
}
