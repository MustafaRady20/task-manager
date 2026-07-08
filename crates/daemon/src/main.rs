use chrono::Local;
use notify_rust::Notification;
use std::time::Duration;
use task_core::db;
use task_core::repository::TaskRepository;

const CHECK_INTERVAL_SECS: u64 = 300;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let pool = db::create_pool().await?;
    let repo = TaskRepository::new(pool);

    println!("taskd started, checking every {CHECK_INTERVAL_SECS}s");

    let mut interval = tokio::time::interval(Duration::from_secs(CHECK_INTERVAL_SECS));

    loop {
        interval.tick().await;

        let now = Local::now().naive_local();

        match repo.find_unnotified_due(now).await {
            Ok(tasks) => {
                for task in tasks {
                    let body = match task.due_time {
                        Some(t) => format!("Due today at {}", t.format("%H:%M")),
                        None => format!("Due {}", task.due_date),
                    };

                    let result = Notification::new()
                        .summary(&format!("Task due: {}", task.title))
                        .body(&body)
                        .appname("taskd")
                        .show();

                    if let Err(e) = result {
                        eprintln!("failed to show notification for task {}: {e}", task.id);
                        continue;
                    }

                    if let Err(e) = repo.mark_notified(task.id).await {
                        eprintln!("failed to mark task {} as notified: {e}", task.id);
                    }
                }
            }
            Err(e) => {
                eprintln!("check for due tasks failed: {e}");
            }
        }
    }
}
