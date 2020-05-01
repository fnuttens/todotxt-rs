//! This crate is a collection of utilities to manage one's todo.txt file.

#![feature(with_options, try_find)]

mod config;

use std::fs::File;
use std::io::{prelude::*, BufReader, SeekFrom};

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

    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;

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
/// - task is already marked as done
pub fn mark_as_done(id: usize) -> Result<(), String> {
    let file = File::open(TODOTXT_PATH).map_err(|e| e.to_string())?;
    let (offset, task) = locate_task(id, file)?;

    if task.starts_with('x') {
        return Err(String::from("This task is already marked as done"));
    }

    let mut file = File::with_options()
        .read(true)
        .write(true)
        .open(TODOTXT_PATH)
        .map_err(|e| e.to_string())?;

    insert_at("x ", offset, &mut file)
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

fn locate_task<T: Read>(id: usize, data: T) -> Result<(usize, String), String> {
    let reader = BufReader::new(data);
    let mut byte_offset = 0;

    let task_search_result = reader
        .lines()
        .enumerate()
        .try_find(|(i, line)| match line {
            Ok(line_value) => {
                let line_nth = i + 1;
                let is_task_located = line_nth == id;

                if !is_task_located {
                    byte_offset += line_value.as_bytes().len() + 1;
                }

                Ok(is_task_located)
            }
            Err(e) => Err(e.to_string()),
        })?;

    let (_, located_task) = task_search_result.ok_or(String::from("Unable to find task"))?;
    located_task
        .map(|task| (byte_offset, task))
        .map_err(|e| e.to_string())
}

fn insert_at<T: Read + Seek + Write>(
    text: &str,
    position: usize,
    data: &mut T,
) -> Result<(), String> {
    data.seek(SeekFrom::Start(position as u64))
        .map_err(|e| e.to_string())?;

    let mut remaining = Vec::new();
    data.read_to_end(&mut remaining)
        .map_err(|e| e.to_string())?;

    data.seek(SeekFrom::Start(position as u64))
        .map_err(|e| e.to_string())?;

    let remaining: Vec<u8> = text
        .as_bytes()
        .iter()
        .chain(remaining.as_slice().iter())
        .map(|character| *character)
        .collect();
    data.write(remaining.as_slice())
        .map_err(|e| e.to_string())?;

    Ok(())
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
        let buf = Cursor::new(b"One\nTwo\nThree\nFour\n");
        assert_eq!(Ok((0, String::from("One"))), locate_task(1, buf.clone()));
        assert_eq!(Ok((8, String::from("Three"))), locate_task(3, buf.clone()));
        assert_eq!(Ok((14, String::from("Four"))), locate_task(4, buf.clone()));
        assert_eq!(
            Err(String::from("Unable to find task")),
            locate_task(5, buf.clone())
        );
        assert_eq!(
            Err(String::from("Unable to find task")),
            locate_task(6, buf)
        );
    }

    #[test]
    fn insert_text_at_specified_location() {
        let mut buf = Cursor::new(b"One\nTwo\n".to_vec());
        assert_eq!(Ok(()), insert_at("Three", 8, &mut buf));
        assert_eq!(b"One\nTwo\nThree", buf.get_ref().as_slice());
        assert_eq!(Ok(()), insert_at("Two and a half\n", 8, &mut buf));
        assert_eq!(b"One\nTwo\nTwo and a half\nThree", buf.get_ref().as_slice());
    }
}
