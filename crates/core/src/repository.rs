use crate::error::RepositoryError;
use crate::models::{Task, TaskNote};
use anyhow::Ok;
use sqlx::MySqlPool;

pub struct TaskRepository {
    pool: MySqlPool,
}

impl TaskRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_task(
        &self,
        title: &str,
        priority: i8,
        due_date: chrono::NaiveDate,
    ) -> anyhow::Result<i64> {
        let result = sqlx::query!(
            r#"
            INSERT INTO tasks (title, status, priority, due_date)
            VALUES (?, 'pending', ?, ?)
            "#,
            title,
            priority,
            due_date,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i64)
    }

    pub async fn find_task_by_id(&self, task_id: i64) -> anyhow::Result<Option<Task>> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT * FROM tasks WHERE id = ?
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn list_by_day(&self, day: chrono::NaiveDate) -> anyhow::Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT * FROM tasks WHERE due_date = ? AND status = 'pending'
            "#,
        )
        .bind(day)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    pub async fn list_all_pending(&self) -> anyhow::Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT * FROM tasks WHERE status = 'pending'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    pub async fn mark_done(&self, task_id: i64) -> anyhow::Result<Task> {
        let result = sqlx::query!(r#"UPDATE tasks SET status = 'done' WHERE id = ?"#, task_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::TaskNotFound(task_id).into());
        }

        Ok(self
            .find_task_by_id(task_id)
            .await?
            .ok_or(RepositoryError::TaskNotFound(task_id))?)
    }

    pub async fn reschedule(
        &self,
        task_id: i64,
        new_date: chrono::NaiveDate,
    ) -> anyhow::Result<Option<Task>> {
        sqlx::query(r#"UPDATE tasks SET due_date = ?, notified_at = NULL WHERE id = ?"#)
            .bind(new_date)
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        self.find_task_by_id(task_id).await
    }

    pub async fn add_note(&self, task_id: i64, content: &str) -> anyhow::Result<i64> {
        let result = sqlx::query!(
            r#"INSERT INTO task_notes (task_id, content) VALUES (?, ?)"#,
            task_id,
            content,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i64)
    }

    pub async fn get_task_with_notes(
        &self,
        task_id: i64,
    ) -> anyhow::Result<Option<(Task, Vec<TaskNote>)>> {
        let Some(task) = self.find_task_by_id(task_id).await? else {
            return Ok(None);
        };

        let notes = sqlx::query_as::<_, TaskNote>(
            r#"
            SELECT * FROM task_notes WHERE task_id = ? ORDER BY created_at
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some((task, notes)))
    }

    pub async fn delete_task(&self, task_id: i64) -> anyhow::Result<()> {
        sqlx::query(r#"DELETE FROM tasks WHERE id = ?"#)
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn find_unnotified_due(
        &self,
        now: chrono::NaiveDateTime,
    ) -> anyhow::Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT * FROM tasks
            WHERE status = 'pending'
              AND notified_at IS NULL
              AND TIMESTAMP(due_date, COALESCE(due_time, '00:00:00')) <= ?
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    pub async fn mark_notified(&self, task_id: i64) -> anyhow::Result<()> {
        sqlx::query!(
            r#"UPDATE tasks SET notified_at = NOW() WHERE id = ?"#,
            task_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
