use chrono::NaiveDate;
use task_core::db;
use task_core::error::RepositoryError;
use task_core::models::TaskStatus;
use task_core::repository::TaskRepository;

const MISSING_TASK_ID: i64 = 999_999_999;

async fn setup() -> TaskRepository {
    dotenvy::dotenv().ok();
    let pool = db::create_pool()
        .await
        .expect("failed to connect to test database (is docker-compose up?)");
    TaskRepository::new(pool)
}

fn date(day_offset: i64) -> NaiveDate {
    NaiveDate::from_ymd_opt(2999, 1, 1).unwrap() + chrono::Duration::days(day_offset)
}

#[tokio::test]
async fn create_and_find_task() {
    let repo = setup().await;
    let due = date(1);

    let id = repo
        .create_task("write tests", 3, due)
        .await
        .expect("create_task failed");

    let task = repo
        .find_task_by_id(id)
        .await
        .expect("find_task_by_id failed")
        .expect("task should exist");

    assert_eq!(task.id, id);
    assert_eq!(task.title, "write tests");
    assert_eq!(task.priority, 3);
    assert_eq!(task.due_date, due);
    assert_eq!(task.status, TaskStatus::Pending);

    repo.delete_task(id).await.unwrap();
}

#[tokio::test]
async fn find_task_by_id_returns_none_for_missing_task() {
    let repo = setup().await;

    let task = repo
        .find_task_by_id(MISSING_TASK_ID)
        .await
        .expect("find_task_by_id failed");

    assert!(task.is_none());
}

#[tokio::test]
async fn list_by_day_returns_only_pending_tasks_for_that_day() {
    let repo = setup().await;
    let due = date(2);

    let pending_id = repo.create_task("due today", 1, due).await.unwrap();
    let done_id = repo.create_task("done today", 1, due).await.unwrap();
    let other_day_id = repo
        .create_task("due tomorrow", 1, date(3))
        .await
        .unwrap();

    repo.mark_done(done_id).await.unwrap();

    let tasks = repo.list_by_day(due).await.expect("list_by_day failed");
    let ids: Vec<i64> = tasks.iter().map(|t| t.id).collect();

    assert!(ids.contains(&pending_id));
    assert!(!ids.contains(&done_id));
    assert!(!ids.contains(&other_day_id));

    repo.delete_task(pending_id).await.unwrap();
    repo.delete_task(done_id).await.unwrap();
    repo.delete_task(other_day_id).await.unwrap();
}

#[tokio::test]
async fn list_all_pending_includes_pending_and_excludes_done() {
    let repo = setup().await;

    let pending_id = repo.create_task("still pending", 1, date(4)).await.unwrap();
    let done_id = repo.create_task("finished", 1, date(4)).await.unwrap();
    repo.mark_done(done_id).await.unwrap();

    let tasks = repo.list_all_pending().await.expect("list_all_pending failed");
    let ids: Vec<i64> = tasks.iter().map(|t| t.id).collect();

    assert!(ids.contains(&pending_id));
    assert!(!ids.contains(&done_id));

    repo.delete_task(pending_id).await.unwrap();
    repo.delete_task(done_id).await.unwrap();
}

#[tokio::test]
async fn mark_done_updates_status() {
    let repo = setup().await;
    let id = repo.create_task("finish me", 1, date(5)).await.unwrap();

    let task = repo.mark_done(id).await.expect("mark_done failed");

    assert_eq!(task.id, id);
    assert_eq!(task.status, TaskStatus::Done);

    repo.delete_task(id).await.unwrap();
}

#[tokio::test]
async fn mark_done_missing_task_returns_error() {
    let repo = setup().await;

    let err = repo
        .mark_done(MISSING_TASK_ID)
        .await
        .expect_err("mark_done should fail for a missing task");

    match err.downcast_ref::<RepositoryError>() {
        Some(RepositoryError::TaskNotFound(id)) => assert_eq!(*id, MISSING_TASK_ID),
        other => panic!("expected RepositoryError::TaskNotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn reschedule_updates_due_date() {
    let repo = setup().await;
    let id = repo.create_task("move me", 1, date(6)).await.unwrap();
    let new_due = date(7);

    let task = repo
        .reschedule(id, new_due)
        .await
        .expect("reschedule failed")
        .expect("task should exist");

    assert_eq!(task.due_date, new_due);

    repo.delete_task(id).await.unwrap();
}

#[tokio::test]
async fn reschedule_missing_task_returns_none() {
    let repo = setup().await;

    let task = repo
        .reschedule(MISSING_TASK_ID, date(8))
        .await
        .expect("reschedule failed");

    assert!(task.is_none());
}

#[tokio::test]
async fn add_note_and_get_task_with_notes() {
    let repo = setup().await;
    let id = repo.create_task("with notes", 1, date(9)).await.unwrap();

    repo.add_note(id, "first note").await.unwrap();
    repo.add_note(id, "second note").await.unwrap();

    let (task, notes) = repo
        .get_task_with_notes(id)
        .await
        .expect("get_task_with_notes failed")
        .expect("task should exist");

    assert_eq!(task.id, id);
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].content, "first note");
    assert_eq!(notes[1].content, "second note");
    assert!(notes.iter().all(|n| n.task_id == id));

    repo.delete_task(id).await.unwrap();
}

#[tokio::test]
async fn get_task_with_notes_returns_none_for_missing_task() {
    let repo = setup().await;

    let result = repo
        .get_task_with_notes(MISSING_TASK_ID)
        .await
        .expect("get_task_with_notes failed");

    assert!(result.is_none());
}

#[tokio::test]
async fn delete_task_removes_task_and_cascades_notes() {
    let repo = setup().await;
    let id = repo.create_task("temporary", 1, date(10)).await.unwrap();
    repo.add_note(id, "will be deleted too").await.unwrap();

    repo.delete_task(id).await.expect("delete_task failed");

    let task = repo.find_task_by_id(id).await.unwrap();
    assert!(task.is_none());

    let with_notes = repo.get_task_with_notes(id).await.unwrap();
    assert!(with_notes.is_none());
}
