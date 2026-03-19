use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubColumn {
    Inbox,
    Done,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub due_date: Option<String>,
    pub is_completed: Option<bool>,
    pub subtasks: Option<Vec<SubtaskResponse>>,
    pub tags: Option<Vec<TagResponse>>,
    pub assignee_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtaskResponse {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionResponse {
    pub completed: TodoResponse,
    pub next: Option<TodoResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardResponse {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnResponse {
    pub id: String,
    pub title: String,
    pub order: Option<i32>,
    pub wip_limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardResponse {
    pub id: String,
    pub todo_id: String,
    pub sub_column: Option<SubColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagResponse {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub todos: Option<Vec<TodoResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_todo() {
        let json = r#"{
            "id": "abc12345-1234-1234-1234-123456789abc",
            "title": "Buy milk",
            "description": "From the store",
            "priority": "High",
            "dueDate": "2024-01-15T00:00:00Z",
            "isCompleted": false,
            "subtasks": [{"id": "sub1", "title": "Check price"}],
            "tags": [{"id": "t1", "name": "shopping"}]
        }"#;
        let todo: TodoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(todo.id, "abc12345-1234-1234-1234-123456789abc");
        assert_eq!(todo.title, "Buy milk");
        assert_eq!(todo.description.as_deref(), Some("From the store"));
        assert_eq!(todo.priority, Some(Priority::High));
        assert_eq!(todo.is_completed, Some(false));
        assert_eq!(todo.subtasks.as_ref().unwrap().len(), 1);
        assert_eq!(todo.tags.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_deserialize_todo_minimal() {
        let json = r#"{"id": "abc", "title": "Minimal"}"#;
        let todo: TodoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(todo.id, "abc");
        assert_eq!(todo.title, "Minimal");
        assert!(todo.priority.is_none());
        assert!(todo.due_date.is_none());
        assert!(todo.is_completed.is_none());
    }

    #[test]
    fn test_deserialize_completion_response() {
        let json = r#"{
            "completed": {"id": "a1", "title": "Done task"},
            "next": {"id": "a2", "title": "Next occurrence", "dueDate": "2024-02-15T00:00:00Z"}
        }"#;
        let resp: CompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.completed.title, "Done task");
        assert!(resp.next.is_some());
        assert_eq!(resp.next.unwrap().title, "Next occurrence");
    }

    #[test]
    fn test_deserialize_completion_no_next() {
        let json = r#"{"completed": {"id": "a1", "title": "Done task"}, "next": null}"#;
        let resp: CompletionResponse = serde_json::from_str(json).unwrap();
        assert!(resp.next.is_none());
    }

    #[test]
    fn test_deserialize_board() {
        let json = r#"{"id": "b1", "title": "My Board"}"#;
        let board: BoardResponse = serde_json::from_str(json).unwrap();
        assert_eq!(board.title, "My Board");
    }

    #[test]
    fn test_deserialize_column() {
        let json = r#"{"id": "c1", "title": "In Progress", "order": 2, "wipLimit": 5}"#;
        let col: ColumnResponse = serde_json::from_str(json).unwrap();
        assert_eq!(col.title, "In Progress");
        assert_eq!(col.order, Some(2));
        assert_eq!(col.wip_limit, Some(5));
    }

    #[test]
    fn test_deserialize_card() {
        let json = r#"{"id": "card1", "todoId": "todo1", "subColumn": "Inbox"}"#;
        let card: CardResponse = serde_json::from_str(json).unwrap();
        assert_eq!(card.todo_id, "todo1");
        assert_eq!(card.sub_column, Some(SubColumn::Inbox));
    }

    #[test]
    fn test_deserialize_card_done() {
        let json = r#"{"id": "card1", "todoId": "todo1", "subColumn": "Done"}"#;
        let card: CardResponse = serde_json::from_str(json).unwrap();
        assert_eq!(card.sub_column, Some(SubColumn::Done));
    }

    #[test]
    fn test_deserialize_project() {
        let json = r#"{
            "id": "p1",
            "title": "My Project",
            "description": "A project",
            "todos": [{"id": "t1", "title": "Task 1"}]
        }"#;
        let proj: ProjectResponse = serde_json::from_str(json).unwrap();
        assert_eq!(proj.title, "My Project");
        assert_eq!(proj.todos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_deserialize_profile() {
        let json = r#"{"id": "user123"}"#;
        let profile: ProfileResponse = serde_json::from_str(json).unwrap();
        assert_eq!(profile.id, "user123");
    }

    #[test]
    fn test_priority_variants() {
        assert_eq!(
            serde_json::from_str::<Priority>(r#""High""#).unwrap(),
            Priority::High
        );
        assert_eq!(
            serde_json::from_str::<Priority>(r#""Medium""#).unwrap(),
            Priority::Medium
        );
        assert_eq!(
            serde_json::from_str::<Priority>(r#""Low""#).unwrap(),
            Priority::Low
        );
    }

    #[test]
    fn test_sub_column_variants() {
        assert_eq!(
            serde_json::from_str::<SubColumn>(r#""Inbox""#).unwrap(),
            SubColumn::Inbox
        );
        assert_eq!(
            serde_json::from_str::<SubColumn>(r#""Done""#).unwrap(),
            SubColumn::Done
        );
        assert_eq!(
            serde_json::from_str::<SubColumn>(r#""Blocked""#).unwrap(),
            SubColumn::Blocked
        );
    }
}
