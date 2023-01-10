use std::{fmt::Display, path::PathBuf};

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, ValueEnum, Clone, Copy)]
pub enum SortMode {
    /// Sort by urgency (least urgent first)
    UrgencyAscending,
    /// Sort by urgency (most urgent first)
    #[default]
    UrgencyDescending,
    /// Sort by the number of days left (nearest due date first)
    DaysLeftAscending,
    /// Sort by the number of days left (nearest due date last)
    DaysLeftDescending,
    /// Sort by the item's name (Ascending)
    NameAscending,
    /// Sort by the item's name (Descending)
    NameDescending,
}

#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, ValueEnum, Serialize, Deserialize,
)]
pub enum Urgency {
    #[default]
    Low,
    Medium,
    High,
}

impl Display for Urgency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Urgency::Low => " ",
                Urgency::Medium => "",
                Urgency::High => "",
            }
        )
    }
}

#[derive(Debug, Args, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Task {
    /// Name of the task (subtasks can be created by namig them task/subtask)
    pub name: PathBuf,
    /// Description of the task
    pub description: Option<String>,
    /// Due date of the task
    #[arg(short, long, value_parser = parse_date)]
    pub due: Option<DateTime<Utc>>,
    /// Urgency of the task
    #[arg(short, long, value_enum, default_value_t)]
    pub urgency: Urgency,
    /// Whether the task has been completed or not
    #[arg(short, long)]
    pub completed: bool,
}

impl Task {
    pub fn new(name: impl AsRef<std::path::Path>) -> Self {
        let name: &std::path::Path = name.as_ref();

        Self {
            name: name.to_path_buf(),
            description: Default::default(),
            due: Default::default(),
            urgency: Default::default(),
            completed: Default::default(),
        }
    }

    pub fn with_due_date(mut self, due: DateTime<Utc>) -> Self {
        self.due = Some(due);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_ugency(mut self, urgency: Urgency) -> Self {
        self.urgency = urgency;
        self
    }

    pub fn complete(mut self) -> Self {
        self.completed = true;
        self
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "- [{}] {}",
            if self.completed { 'x' } else { ' ' },
            self.urgency
        )?;

        if let Some(date) = self.due {
            write!(f, " {date}")?;
        }

        write!(f, " {}", self.name.as_path().as_os_str().to_string_lossy())?;

        if let Some(desc) = &self.description {
            write!(f, "\n    {desc}")?;
        }

        Ok(())
    }
}

fn sort_tasks_due_date(tasks: Vec<Task>) -> Vec<Task> {
    let (mut with_date, without_date): (Vec<_>, Vec<_>) =
        tasks.into_iter().partition(|task| task.due.is_some());
    with_date.sort_by_key(|task| task.due.expect("partition with due dates"));
    with_date.extend(without_date);
    with_date
}

fn sort_tasks_name(tasks: &mut [Task]) {
    tasks.sort_by_key(|task| task.name.clone());
}

fn sort_tasks_urgency(tasks: &mut [Task]) {
    tasks.sort_by_key(|task| task.urgency);
}

pub fn sort_tasks(tasks: Vec<Task>, mode: SortMode) -> Vec<Task> {
    match mode {
        SortMode::UrgencyAscending => {
            let mut sorted = tasks;
            sort_tasks_name(&mut sorted);
            let mut sorted = sort_tasks_due_date(sorted);
            sort_tasks_urgency(&mut sorted);
            sorted
        }
        SortMode::UrgencyDescending => {
            let mut sorted = tasks;
            sort_tasks_name(&mut sorted);
            let mut sorted = sort_tasks_due_date(sorted);
            sorted.reverse();
            sort_tasks_urgency(&mut sorted);
            sorted.reverse();
            sorted
        }
        SortMode::DaysLeftAscending => {
            let mut sorted = tasks;
            sort_tasks_name(&mut sorted);
            sorted.reverse();
            sort_tasks_urgency(&mut sorted);
            sorted.reverse();
            sort_tasks_due_date(sorted)
        }
        SortMode::DaysLeftDescending => {
            let mut sorted = tasks;
            sort_tasks_name(&mut sorted);
            sorted.reverse();
            sort_tasks_urgency(&mut sorted);
            let mut sorted = sort_tasks_due_date(sorted);
            sorted.reverse();
            sorted
        }
        SortMode::NameAscending => {
            let mut sorted = sort_tasks_due_date(tasks);
            sorted.reverse();
            sort_tasks_urgency(&mut sorted);
            sorted.reverse();
            sort_tasks_name(&mut sorted);
            sorted
        }
        SortMode::NameDescending => {
            let mut sorted = sort_tasks_due_date(tasks);
            sorted.reverse();
            sort_tasks_urgency(&mut sorted);
            sort_tasks_name(&mut sorted);
            sorted.reverse();
            sorted
        }
    }
}

fn parse_date(date: &str) -> std::io::Result<DateTime<Utc>> {
    let today = Local::now();

    if let Ok(time) = date.parse::<NaiveTime>() {
        return Ok(today
            .date_naive()
            .and_time(time)
            .and_local_timezone(Local)
            .earliest()
            .expect("valid date")
            .into());
    }

    if let Ok(date) = date.parse::<NaiveDate>() {
        return Ok(date
            .and_time(
                NaiveTime::from_num_seconds_from_midnight_opt(0, 0).expect("midnight is valid"),
            )
            .and_local_timezone(Local)
            .earliest()
            .expect("valid date")
            .into());
    }

    if let Ok(datetime) = date.parse::<NaiveDateTime>() {
        return Ok(datetime
            .and_local_timezone(Local)
            .earliest()
            .expect("valid date")
            .into());
    }

    todo!("parse {date} as datetime")
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{tasks::sort_tasks, Task, Urgency};

    #[test]
    fn test_task_name_sorting_asc() {
        let tasks = vec![
            Task::new("c").with_ugency(Urgency::Medium),
            Task::new("b").with_ugency(Urgency::High),
            Task::new("b").with_ugency(Urgency::Low),
            Task::new("a").with_ugency(Urgency::Medium),
            Task::new("a"),
        ];
        let expect = vec![
            Task::new("a").with_ugency(Urgency::Medium),
            Task::new("a"),
            Task::new("b").with_ugency(Urgency::High),
            Task::new("b").with_ugency(Urgency::Low),
            Task::new("c").with_ugency(Urgency::Medium),
        ];
        let tasks = sort_tasks(tasks, crate::SortMode::NameAscending);
        assert_eq!(tasks, expect);
    }

    #[test]
    fn test_task_name_sorting_des() {
        let tasks = vec![
            Task::new("a").with_ugency(Urgency::Medium),
            Task::new("a"),
            Task::new("b").with_ugency(Urgency::High),
            Task::new("b").with_ugency(Urgency::Low),
            Task::new("c").with_ugency(Urgency::Medium),
        ];
        let expect = vec![
            Task::new("c").with_ugency(Urgency::Medium),
            Task::new("b").with_ugency(Urgency::High),
            Task::new("b").with_ugency(Urgency::Low),
            Task::new("a").with_ugency(Urgency::Medium),
            Task::new("a"),
        ];
        let tasks = sort_tasks(tasks, crate::SortMode::NameDescending);
        assert_eq!(tasks, expect);
    }
}
