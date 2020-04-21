mod config;

use std::fs::OpenOptions;
use std::io::{prelude::*, BufReader};

use chrono::{NaiveDate, Utc};

use config::TODOTXT_PATH;

pub fn add(
    todo: &str,
    priority: Option<char>,
    creation_date: Option<NaiveDate>,
    insert_creation_date: bool,
) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(TODOTXT_PATH)
        .map_err(|e| e.to_string())?;

    let new_todo = format_task(todo, priority, creation_date, insert_creation_date);
    writeln!(&file, "{}", new_todo).map_err(|e| e.to_string())?;

    file.seek(std::io::SeekFrom::Start(0))
        .map_err(|e| e.to_string())?;

    let todo_id = BufReader::new(file).lines().count();
    println!("{}: {}", todo_id, new_todo);

    Ok(())
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

#[cfg(test)]
mod should {
    use super::*;

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
}
