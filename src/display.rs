use colored::Colorize;

use crate::models::{Priority, SubColumn, TodoResponse};

pub fn short_id(id: &str) -> &str {
    id.split('-').next().unwrap_or(id)
}

pub fn priority_icon(priority: &Option<Priority>) -> String {
    match priority {
        Some(Priority::High) => format!("{}", "●".red()),
        Some(Priority::Medium) => format!("{}", "●".yellow()),
        Some(Priority::Low) => format!("{}", "●".green()),
        None => "○".to_string(),
    }
}

pub fn status_icon(completed: bool) -> String {
    if completed {
        format!("{}", "✓".green())
    } else {
        "○".to_string()
    }
}

pub fn sub_column_icon(sub: &SubColumn) -> String {
    match sub {
        SubColumn::Inbox => "○".to_string(),
        SubColumn::Done => format!("{}", "✓".green()),
        SubColumn::Blocked => format!("{}", "⊗".red()),
    }
}

fn format_due(due_date: &Option<String>) -> Option<String> {
    match due_date {
        Some(d) if d != "null" && !d.is_empty() => {
            let date_part = d.split('T').next().unwrap_or(d);
            Some(date_part.to_string())
        }
        _ => None,
    }
}

pub fn format_todo_line(todo: &TodoResponse) -> String {
    let sid = short_id(&todo.id);
    let completed = todo.is_completed.unwrap_or(false);

    if completed {
        let icon = status_icon(true);
        let dimmed_priority = format!("{}", "○".dimmed());
        let _ = dimmed_priority; // we use status_icon for completed
        match format_due(&todo.due_date) {
            Some(due) => {
                format!(
                    "  {} {}  {}  {}",
                    icon,
                    sid.dimmed(),
                    due.dimmed(),
                    todo.title.dimmed().strikethrough()
                )
            }
            None => {
                format!(
                    "  {} {}  {}",
                    icon,
                    sid.dimmed(),
                    todo.title.dimmed().strikethrough()
                )
            }
        }
    } else {
        let icon = priority_icon(&todo.priority);
        match format_due(&todo.due_date) {
            Some(due) => format!("  {} {}  {}  {}", icon, sid, due, todo.title),
            None => format!("  {} {}  {}", icon, sid, todo.title),
        }
    }
}

pub fn format_todo_detail(todo: &TodoResponse) -> String {
    let mut lines = Vec::new();
    lines.push(format!("{}", todo.title.blue()));

    if let Some(ref desc) = todo.description {
        if !desc.is_empty() {
            lines.push(format!("   {}", desc));
        }
    }

    if let Some(due) = format_due(&todo.due_date) {
        lines.push(format!("   Due: {}", due));
    }

    lines.push(String::new());

    // Subtasks
    if let Some(ref subtasks) = todo.subtasks {
        if !subtasks.is_empty() {
            lines.push(format!("   {}", "Subtasks:".green()));
            for st in subtasks {
                lines.push(format!("   ○ {}  {}", short_id(&st.id), st.title));
            }
        }
    }

    // Tags
    if let Some(ref tags) = todo.tags {
        if !tags.is_empty() {
            lines.push(String::new());
            let tag_str: Vec<String> = tags.iter().map(|t| format!("[{}]", t.name)).collect();
            lines.push(format!("Tags: {}", tag_str.join(" ")));
        }
    }

    lines.join("\n")
}

pub fn format_project_line(id: &str, title: &str, description: &Option<String>) -> String {
    let sid = short_id(id);
    match description {
        Some(desc) if !desc.is_empty() => format!("  📁 {}  {} - {}", sid, title, desc),
        _ => format!("  📁 {}  {}", sid, title),
    }
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_id() {
        assert_eq!(short_id("abc12345-1234-1234-1234-123456789abc"), "abc12345");
        assert_eq!(short_id("nohyphen"), "nohyphen");
    }

    #[test]
    fn test_priority_icon_variants() {
        // Just test that they return non-empty strings
        assert!(!priority_icon(&Some(Priority::High)).is_empty());
        assert!(!priority_icon(&Some(Priority::Medium)).is_empty());
        assert!(!priority_icon(&Some(Priority::Low)).is_empty());
        assert_eq!(priority_icon(&None), "○");
    }

    #[test]
    fn test_status_icon() {
        assert!(status_icon(true).contains('✓'));
        assert_eq!(status_icon(false), "○");
    }

    #[test]
    fn test_sub_column_icon() {
        assert_eq!(sub_column_icon(&SubColumn::Inbox), "○");
        assert!(sub_column_icon(&SubColumn::Done).contains('✓'));
        assert!(sub_column_icon(&SubColumn::Blocked).contains('⊗'));
    }

    #[test]
    fn test_format_todo_line_pending_no_due() {
        let todo = TodoResponse {
            id: "abc12345-xxxx".to_string(),
            title: "Buy milk".to_string(),
            description: None,
            priority: Some(Priority::High),
            due_date: None,
            is_completed: Some(false),
            subtasks: None,
            tags: None,
            assignee_user_id: None,
        };
        let line = format_todo_line(&todo);
        assert!(line.contains("abc12345"));
        assert!(line.contains("Buy milk"));
    }

    #[test]
    fn test_format_todo_line_pending_with_due() {
        let todo = TodoResponse {
            id: "abc12345-xxxx".to_string(),
            title: "Buy milk".to_string(),
            description: None,
            priority: None,
            due_date: Some("2024-01-15T00:00:00Z".to_string()),
            is_completed: Some(false),
            subtasks: None,
            tags: None,
            assignee_user_id: None,
        };
        let line = format_todo_line(&todo);
        assert!(line.contains("2024-01-15"));
    }

    #[test]
    fn test_format_todo_line_completed() {
        let todo = TodoResponse {
            id: "abc12345-xxxx".to_string(),
            title: "Done task".to_string(),
            description: None,
            priority: Some(Priority::Medium),
            due_date: None,
            is_completed: Some(true),
            subtasks: None,
            tags: None,
            assignee_user_id: None,
        };
        let line = format_todo_line(&todo);
        assert!(line.contains("abc12345"));
    }

    #[test]
    fn test_format_todo_detail() {
        let todo = TodoResponse {
            id: "abc12345-xxxx".to_string(),
            title: "Buy milk".to_string(),
            description: Some("From the store".to_string()),
            priority: Some(Priority::High),
            due_date: Some("2024-01-15T00:00:00Z".to_string()),
            is_completed: Some(false),
            subtasks: Some(vec![crate::models::SubtaskResponse {
                id: "sub1-xxxx".to_string(),
                title: "Check price".to_string(),
            }]),
            tags: Some(vec![crate::models::TagResponse {
                id: Some("t1".to_string()),
                name: "shopping".to_string(),
            }]),
            assignee_user_id: None,
        };
        let detail = format_todo_detail(&todo);
        assert!(detail.contains("From the store"));
        assert!(detail.contains("2024-01-15"));
        assert!(detail.contains("Check price"));
        assert!(detail.contains("[shopping]"));
    }

    #[test]
    fn test_format_project_line_with_desc() {
        let line = format_project_line("p1-xxxx", "My Project", &Some("A description".to_string()));
        assert!(line.contains("p1"));
        assert!(line.contains("My Project"));
        assert!(line.contains("A description"));
    }

    #[test]
    fn test_format_project_line_without_desc() {
        let line = format_project_line("p1-xxxx", "My Project", &None);
        assert!(line.contains("p1"));
        assert!(line.contains("My Project"));
        assert!(!line.contains(" - "));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world this is long", 10), "hello w...");
    }
}
