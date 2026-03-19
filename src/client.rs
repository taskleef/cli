use async_trait::async_trait;
use serde_json::Value;

use crate::error::{Result, TaskleefError};
use crate::models::*;

#[async_trait]
pub trait ApiClient: Send + Sync {
    // Todos
    async fn list_todos(&self) -> Result<Vec<TodoResponse>>;
    async fn get_todo(&self, id: &str) -> Result<TodoResponse>;
    async fn create_todo(&self, title: &str) -> Result<TodoResponse>;
    async fn complete_todo(&self, id: &str) -> Result<CompletionResponse>;
    async fn delete_todo(&self, id: &str) -> Result<TodoResponse>;
    async fn update_todo(&self, id: &str, body: Value) -> Result<TodoResponse>;
    async fn list_inbox(&self) -> Result<Vec<TodoResponse>>;

    // Subtasks
    async fn create_subtask(&self, parent_id: &str, title: &str) -> Result<TodoResponse>;

    // Projects
    async fn list_projects(&self) -> Result<Vec<ProjectResponse>>;
    async fn get_project(&self, id: &str) -> Result<ProjectResponse>;
    async fn create_project(&self, title: &str) -> Result<ProjectResponse>;
    async fn delete_project(&self, id: &str) -> Result<ProjectResponse>;
    async fn add_todo_to_project(&self, project_id: &str, todo_id: &str) -> Result<ProjectResponse>;
    async fn remove_todo_from_project(&self, project_id: &str, todo_id: &str) -> Result<ProjectResponse>;

    // Boards
    async fn list_boards(&self) -> Result<Vec<BoardResponse>>;
    async fn get_board(&self, id: &str) -> Result<BoardResponse>;
    async fn list_columns(&self, board_id: &str) -> Result<Vec<ColumnResponse>>;
    async fn list_cards(&self, column_id: &str) -> Result<Vec<CardResponse>>;
    async fn update_card(&self, card_id: &str, body: Value) -> Result<CardResponse>;
    async fn delete_card(&self, card_id: &str) -> Result<()>;

    // Profile
    async fn get_profile(&self) -> Result<ProfileResponse>;
}

pub struct HttpApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl HttpApiClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(TaskleefError::Api(text));
        }

        Ok(resp.json().await?)
    }

    async fn post<T: serde::de::DeserializeOwned>(&self, path: &str, body: Value) -> Result<T> {
        let resp = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(TaskleefError::Api(text));
        }

        Ok(resp.json().await?)
    }

    async fn put<T: serde::de::DeserializeOwned>(&self, path: &str, body: Value) -> Result<T> {
        let resp = self
            .client
            .put(format!("{}{}", self.base_url, path))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(TaskleefError::Api(text));
        }

        Ok(resp.json().await?)
    }

    async fn patch<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .client
            .patch(format!("{}{}", self.base_url, path))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(TaskleefError::Api(text));
        }

        Ok(resp.json().await?)
    }

    async fn delete_req<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .client
            .delete(format!("{}{}", self.base_url, path))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(TaskleefError::Api(text));
        }

        Ok(resp.json().await?)
    }

    async fn delete_no_body(&self, path: &str) -> Result<()> {
        let resp = self
            .client
            .delete(format!("{}{}", self.base_url, path))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(TaskleefError::Api(text));
        }

        Ok(())
    }
}

#[async_trait]
impl ApiClient for HttpApiClient {
    async fn list_todos(&self) -> Result<Vec<TodoResponse>> {
        self.get("/api/todos").await
    }

    async fn get_todo(&self, id: &str) -> Result<TodoResponse> {
        self.get(&format!("/api/todos/{}", id)).await
    }

    async fn create_todo(&self, title: &str) -> Result<TodoResponse> {
        self.post("/api/todos", serde_json::json!({"title": title}))
            .await
    }

    async fn complete_todo(&self, id: &str) -> Result<CompletionResponse> {
        self.patch(&format!("/api/todos/{}/complete", id)).await
    }

    async fn delete_todo(&self, id: &str) -> Result<TodoResponse> {
        self.delete_req(&format!("/api/todos/{}", id)).await
    }

    async fn update_todo(&self, id: &str, body: Value) -> Result<TodoResponse> {
        self.put(&format!("/api/todos/{}", id), body).await
    }

    async fn list_inbox(&self) -> Result<Vec<TodoResponse>> {
        self.get("/api/inbox").await
    }

    async fn create_subtask(&self, parent_id: &str, title: &str) -> Result<TodoResponse> {
        self.post(
            &format!("/api/todos/{}/subtasks", parent_id),
            serde_json::json!({"title": title}),
        )
        .await
    }

    async fn list_projects(&self) -> Result<Vec<ProjectResponse>> {
        self.get("/api/projects").await
    }

    async fn get_project(&self, id: &str) -> Result<ProjectResponse> {
        self.get(&format!("/api/projects/{}", id)).await
    }

    async fn create_project(&self, title: &str) -> Result<ProjectResponse> {
        self.post("/api/projects", serde_json::json!({"title": title}))
            .await
    }

    async fn delete_project(&self, id: &str) -> Result<ProjectResponse> {
        self.delete_req(&format!("/api/projects/{}", id)).await
    }

    async fn add_todo_to_project(&self, project_id: &str, todo_id: &str) -> Result<ProjectResponse> {
        self.post(
            &format!("/api/projects/{}/todos/{}", project_id, todo_id),
            serde_json::json!({}),
        )
        .await
    }

    async fn remove_todo_from_project(&self, project_id: &str, todo_id: &str) -> Result<ProjectResponse> {
        self.delete_req(&format!("/api/projects/{}/todos/{}", project_id, todo_id))
            .await
    }

    async fn list_boards(&self) -> Result<Vec<BoardResponse>> {
        self.get("/api/boards").await
    }

    async fn get_board(&self, id: &str) -> Result<BoardResponse> {
        self.get(&format!("/api/boards/{}", id)).await
    }

    async fn list_columns(&self, board_id: &str) -> Result<Vec<ColumnResponse>> {
        self.get(&format!("/api/boards/{}/columns", board_id)).await
    }

    async fn list_cards(&self, column_id: &str) -> Result<Vec<CardResponse>> {
        self.get(&format!("/api/columns/{}/cards", column_id)).await
    }

    async fn update_card(&self, card_id: &str, body: Value) -> Result<CardResponse> {
        self.put(&format!("/api/cards/{}", card_id), body).await
    }

    async fn delete_card(&self, card_id: &str) -> Result<()> {
        self.delete_no_body(&format!("/api/cards/{}", card_id)).await
    }

    async fn get_profile(&self) -> Result<ProfileResponse> {
        self.get("/api/profile").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup() -> (MockServer, HttpApiClient) {
        let server = MockServer::start().await;
        let client = HttpApiClient::new(&server.uri(), "test-api-key");
        (server, client)
    }

    #[tokio::test]
    async fn test_list_todos() {
        let (server, client) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/todos"))
            .and(header("X-API-Key", "test-api-key"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([
                        {"id": "t1", "title": "Todo 1"},
                        {"id": "t2", "title": "Todo 2"}
                    ])),
            )
            .mount(&server)
            .await;

        let todos = client.list_todos().await.unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "Todo 1");
    }

    #[tokio::test]
    async fn test_create_todo() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/api/todos"))
            .and(header("X-API-Key", "test-api-key"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": "new-id", "title": "New todo"})),
            )
            .mount(&server)
            .await;

        let todo = client.create_todo("New todo").await.unwrap();
        assert_eq!(todo.title, "New todo");
    }

    #[tokio::test]
    async fn test_complete_todo() {
        let (server, client) = setup().await;

        Mock::given(method("PATCH"))
            .and(path("/api/todos/t1/complete"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "completed": {"id": "t1", "title": "Done"},
                    "next": null
                })),
            )
            .mount(&server)
            .await;

        let resp = client.complete_todo("t1").await.unwrap();
        assert_eq!(resp.completed.title, "Done");
        assert!(resp.next.is_none());
    }

    #[tokio::test]
    async fn test_api_error() {
        let (server, client) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/todos"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let result = client.list_todos().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_boards() {
        let (server, client) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/boards"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([{"id": "b1", "title": "Board 1"}])),
            )
            .mount(&server)
            .await;

        let boards = client.list_boards().await.unwrap();
        assert_eq!(boards.len(), 1);
        assert_eq!(boards[0].title, "Board 1");
    }

    #[tokio::test]
    async fn test_list_projects() {
        let (server, client) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/projects"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([{"id": "p1", "title": "Project 1"}])),
            )
            .mount(&server)
            .await;

        let projects = client.list_projects().await.unwrap();
        assert_eq!(projects.len(), 1);
    }

    #[tokio::test]
    async fn test_get_profile() {
        let (server, client) = setup().await;

        Mock::given(method("GET"))
            .and(path("/api/profile"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": "user1"})),
            )
            .mount(&server)
            .await;

        let profile = client.get_profile().await.unwrap();
        assert_eq!(profile.id, "user1");
    }

    #[tokio::test]
    async fn test_delete_card() {
        let (server, client) = setup().await;

        Mock::given(method("DELETE"))
            .and(path("/api/cards/c1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        client.delete_card("c1").await.unwrap();
    }
}
