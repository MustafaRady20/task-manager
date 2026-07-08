#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub status: TaskStatus,
    pub priority: i8,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub due_date: chrono::NaiveDate,
    pub due_time: Option<chrono::NaiveTime>,
    pub notified_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct TaskNote {
    pub id: i64,
    pub task_id: i64,
    pub content: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}
