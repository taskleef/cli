use colored::Colorize;

use crate::client::ApiClient;
use crate::display::short_id;
use crate::error::Result;
use crate::resolve::resolve_todo;

pub async fn add(client: &dyn ApiClient, parent_query: &str, title: &str) -> Result<()> {
    let parent_id = resolve_todo(client, parent_query).await?;
    let subtask = client.create_subtask(&parent_id, title).await?;
    let sid = short_id(&subtask.id);
    println!("{} {} ({})", "Created subtask:".green(), title, sid);
    Ok(())
}
