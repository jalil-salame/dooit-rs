use std::{path::PathBuf, process::Command};

use clap::{Parser, Subcommand};
use dooit_tasks::{dirs, dirs::get_tasks, tasks::sort_tasks, SortMode, Task};

#[derive(Parser, Debug)]
struct Cli {
    /// Editor to use when modifying files
    #[arg(short, long, env)]
    editor: Option<PathBuf>,
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// List tasks
    List {
        /// Sort tasks
        #[arg(short, long, value_enum, default_value_t)]
        sort: SortMode,
        /// Show completed items
        #[arg(short, long)]
        completed: bool,
        /// Show overdue items
        #[arg(short, long)]
        overdue: bool,
    },
    /// Add a task
    Add(Task),
    /// Edit the Configuration
    Config,
}

/// Returns Ok(false) if the path already exists
fn create_dir_all_if_missing(path: impl AsRef<std::path::Path>) -> std::io::Result<bool> {
    let path: &std::path::Path = path.as_ref();

    if path.exists() {
        return Ok(false);
    }

    std::fs::create_dir_all(path).map(|_| true)
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    match args.mode {
        Mode::List {
            sort,
            completed,
            overdue,
        } => {
            let data_dir = dirs::get_data_dir().expect("data dir");

            if !data_dir.exists() {
                panic!(
                    "The task directory is empty, add some by running:\n\t`{} add`",
                    std::env::args().next().unwrap_or_else(|| "dooit-rs".into())
                );
            }

            let tasks = get_tasks()?;
            let today = chrono::Utc::now();
            let filtered = tasks
                .into_iter()
                .filter(|task| {
                    (!task.completed || completed)
                        && (task.due.map(|date| date >= today).unwrap_or(true) || overdue)
                })
                .collect::<Vec<_>>();

            if filtered.is_empty() {
                println!("No tasks to do!");
                return Ok(());
            }

            let sorted = sort_tasks(filtered, sort);

            for task in sorted {
                println!("{task}");
            }
        }
        Mode::Add(task) => {
            let data_dir = dirs::get_data_dir().expect("data dir");

            create_dir_all_if_missing(data_dir)
                .map(|created| {
                    if created {
                        println!("The task directory doesn't exist, creating it...");
                    }
                })
                .expect("create task directory");

            let task_path = {
                let mut task_path = data_dir.join(task.name.as_path());
                task_path.set_extension("toml");
                task_path
            };

            create_dir_all_if_missing(task_path.parent().expect("valid parent"))
                .expect("create subtask folder");

            std::fs::write(task_path, toml::to_vec(&task).expect("valid toml"))
                .expect("write task to file");
        }
        Mode::Config => {
            let config_dir = dirs::get_config_dir().expect("data dir");
            if !config_dir.exists() {
                std::fs::create_dir_all(config_dir).expect("create config dir");
            }

            let config_path = config_dir.join("config.toml");
            if !config_path.exists() {
                std::fs::write(
                    &config_path,
                    "# This is the sample config
",
                )
                .expect("create sample config");
            }

            if let Some(editor) = args.editor {
                Command::new(&editor)
                    .arg(&config_path)
                    .status()
                    .unwrap_or_else(|_| panic!("edit {config_path:?} with {editor:?}"));
            } else {
                panic!("No editor configured, set the EDITOR environment variable or pass it as an argument with --editor")
            }
        }
    }

    Ok(())
}
