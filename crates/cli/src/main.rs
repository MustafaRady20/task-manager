use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use comfy_table::{presets::UTF8_FULL, Table};
use task_core::{
    db,
    models::{Task, TaskStatus},
    repository::TaskRepository,
};

#[derive(Parser)]
#[command(name = "task", about = "Personal task manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Add {
        title: String,
        #[arg(long)]
        due: Option<String>,
        #[arg(long, default_value_t = 0)]
        priority: i8,
    },

    List {
        #[arg(long)]
        day: Option<String>,
        #[arg(long)]
        all: bool,
    },

    Done {
        id: i64,
    },

    Note {
        id: i64,
        content: String,
    },

    Reschedule {
        id: i64,
        new_date: String,
    },

    Show {
        id: i64,
    },

    Rm {
        id: i64,
    },
}

fn parse_date_arg(s: &str) -> anyhow::Result<NaiveDate> {
    match s.to_lowercase().as_str() {
        "today" => Ok(Local::now().date_naive()),
        "tomorrow" => Ok(Local::now().date_naive() + chrono::Duration::days(1)),
        other => NaiveDate::parse_from_str(other, "%Y-%m-%d").map_err(|_| {
            anyhow::anyhow!("invalid date '{other}', expected YYYY-MM-DD or 'today'/'tomorrow'")
        }),
    }
}

fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::Done => "done",
        TaskStatus::Cancelled => "cancelled",
    }
}

fn print_tasks_table(tasks: &[Task]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["ID", "Title", "Due", "Priority", "Status"]);

    for t in tasks {
        table.add_row(vec![
            t.id.to_string(),
            t.title.clone(),
            t.due_date.to_string(),
            t.priority.to_string(),
            status_label(&t.status).to_string(),
        ]);
    }

    println!("{table}");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let pool = db::create_pool().await?;
    let repo = TaskRepository::new(pool);

    match cli.command {
        Command::Add {
            title,
            due,
            // time,
            priority,
        } => {
            let due_date = match due {
                Some(s) => parse_date_arg(&s)?,
                None => Local::now().date_naive(),
            };
            // let due_time = match time {
            //     Some(s) => Some(parse_time_arg(&s)?),
            //     None => None,
            // };

            let id = repo.create_task(&title, priority, due_date).await?;
            println!("Created task #{id}");
        }

        Command::List { day, all } => {
            let tasks = if all {
                repo.list_all_pending().await?
            } else {
                let target_day = match day {
                    Some(s) => parse_date_arg(&s)?,
                    None => Local::now().date_naive(),
                };
                repo.list_by_day(target_day).await?
            };

            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                print_tasks_table(&tasks);
            }
        }

        Command::Done { id } => {
            let task = repo.mark_done(id).await?;
            println!("Marked task #{} as done: {}", task.id, task.title);
        }

        Command::Note { id, content } => {
            let note_id = repo.add_note(id, &content).await?;
            println!("Added note #{note_id} to task #{id}");
        }

        Command::Reschedule { id, new_date } => {
            let parsed = parse_date_arg(&new_date)?;
            match repo.reschedule(id, parsed).await? {
                Some(task) => println!("Rescheduled task #{} to {}", task.id, task.due_date),
                None => println!("No task found with id {id}"),
            }
        }

        Command::Show { id } => match repo.get_task_with_notes(id).await? {
            Some((task, notes)) => {
                println!(
                    "#{} {} [{}]",
                    task.id,
                    task.title,
                    status_label(&task.status)
                );
                println!(
                    "Due: {} {}",
                    task.due_date,
                    task.due_time
                        .map(|t| t.format("%H:%M").to_string())
                        .unwrap_or_default()
                );
                println!("Priority: {}", task.priority);
                if notes.is_empty() {
                    println!("No notes.");
                } else {
                    println!("Notes:");
                    for n in notes {
                        println!(
                            "  [{}] {}",
                            n.created_at.map(|c| c.to_string()).unwrap_or_default(),
                            n.content
                        );
                    }
                }
            }
            None => println!("No task found with id {id}"),
        },

        Command::Rm { id } => {
            repo.delete_task(id).await?;
            println!("Deleted task #{id}");
        }
    }

    Ok(())
}
