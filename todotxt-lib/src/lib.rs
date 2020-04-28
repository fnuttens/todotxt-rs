//! This crate is a collection of utilities to manage one's todo.txt file.

#![feature(with_options, try_find)]

mod config;

use std::fs::File;
use std::io::{prelude::*, BufReader};

use chrono::{NaiveDate, Utc};

use config::TODOTXT_PATH;

/// Adds a task to the list
///
/// The new task will be inserted at the bottom of todo.txt.
/// In case of success, this function returns a tuple containing the task ID and its formatted string.
///
/// # Errors
///
/// - couldn't find nor open the file with read and write access
/// - couldn't write to the file
pub fn add(
    todo: &str,
    priority: Option<char>,
    creation_date: Option<NaiveDate>,
    insert_creation_date: bool,
) -> Result<(usize, String), String> {
    let mut file = File::with_options()
        .read(true)
        .append(true)
        .open(TODOTXT_PATH)
        .map_err(|e| e.to_string())?;

    let new_task = format_task(todo, priority, creation_date, insert_creation_date);
    writeln!(&file, "{}", new_task).map_err(|e| e.to_string())?;

    file.seek(std::io::SeekFrom::Start(0))
        .map_err(|e| e.to_string())?;

    let task_id = BufReader::new(file).lines().count();
    Ok((task_id, new_task))
}

/// Marks a task as accomplished
///
/// A cross ('x') is inserted at the beginning of the task entry to mark it as done.
///
/// # Errors
///
/// - couldn't find task with given ID
pub fn mark_as_done(id: usize) -> Result<(), String> {
    let file = File::open(TODOTXT_PATH).map_err(|e| e.to_string())?;
    let _fulfilled_task = locate_task(id, file)?;

    todo!("Insert a 'x' at the beginning of the task")
}

fn format_task(
    todo: &str,
    priority: Option<char>,
    creation_date: Option<NaiveDate>,
    insert_creation_date: bool,
) -> String {
    let mut task = String::new();

    if let Some(priority_value) = priority {
        let fmt_priority = format!("({}) ", priority_value);
        task.push_str(fmt_priority.as_str());
    }

    if let Some(forced_creation_date) = creation_date {
        let fmt_creation_date = format!("{} ", forced_creation_date);
        task.push_str(fmt_creation_date.as_str());
    } else if insert_creation_date {
        let today = Utc::today();
        let fmt_creation_date = format!("{} ", today.format("%Y-%m-%d"));
        task.push_str(fmt_creation_date.as_str());
    }

    task.push_str(todo);
    task
}

// TODO: return byte offset along with task string
fn locate_task<T: Read>(id: usize, data: T) -> Result<String, String> {
    let reader = BufReader::new(data);

    let mut line_nth: usize = 0;
    let task_search_result = reader.lines().try_find(|line| {
        line_nth += 1;
        match line {
            Ok(_) => Ok(line_nth == id),
            Err(e) => Err(e.to_string()),
        }
    })?;

    task_search_result
        .ok_or(String::from("Unable to find task"))?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod should {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn format_tasks_correctly() {
        assert_eq!("todo", format_task("todo", None, None, false));
        assert_eq!(
            "(A) 2020-05-02 todo",
            format_task(
                "todo",
                Some('A'),
                Some(NaiveDate::parse_from_str("2020-05-02", "%Y-%m-%d").unwrap()),
                false
            )
        );
    }

    #[test]
    fn locate_tasks_correctly() {
        let buf = Cursor::new(b"One\nTwo\nThree\n");
        assert_eq!(Ok(String::from("Two")), locate_task(2, buf.clone()));
        assert_eq!(Err(String::from("Unable to find task")), locate_task(4, buf.clone()));
        assert_eq!(Err(String::from("Unable to find task")), locate_task(5, buf));
    }
}
