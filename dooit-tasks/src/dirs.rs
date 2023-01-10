use std::{ffi::OsStr, path::Path};

use directories::ProjectDirs;
use lazy_static::lazy_static;

use crate::Task;

lazy_static! {
    static ref PROJECT_DIRS: Option<ProjectDirs> = ProjectDirs::from("rs", "salameme", "dooit-rs");
}

fn get_project_dirs() -> Option<&'static ProjectDirs> {
    PROJECT_DIRS.as_ref()
}

pub fn get_config_dir() -> Option<&'static Path> {
    get_project_dirs().map(ProjectDirs::config_dir)
}

pub fn get_data_dir() -> Option<&'static Path> {
    get_project_dirs().map(ProjectDirs::data_dir)
}

pub fn get_tasks() -> std::io::Result<Vec<Task>> {
    let data_dir = get_data_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "data dir not available")
    })?;

    match data_dir.read_dir() {
        Ok(_) => get_tasks_in_dir_recursive(data_dir),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Ok(Vec::new()),
            _ => Err(err),
        },
    }
}

fn get_tasks_in_dir_recursive(dir: &std::path::Path) -> std::io::Result<Vec<Task>> {
    let mut tasks = vec![];

    for file in dir.read_dir()? {
        let file = file?;
        let path = file.path();

        if path.extension() == Some(OsStr::new("toml")) {
            tasks.push(toml::from_slice(&std::fs::read(path)?)?);
            continue;
        }

        if !path.is_dir() {
            continue;
        }

        tasks.extend(get_tasks_in_dir_recursive(&path)?);
    }

    Ok(tasks)
}
