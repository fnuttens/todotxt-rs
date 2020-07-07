//! This crate is a collection of utilities to manage one's todo.txt file.

mod config;

use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, SeekFrom};
use std::vec::Vec;

use chrono::{NaiveDate, Utc};

use config::{DONETXT_PATH, TODOTXT_PATH};

const NEWLINE_BYTE: usize = 1;

pub enum SortFilter {
    Alphabetic,
    Completed,
    CompletionDate,
    Context,
    CreationDate,
    DueDate,
    Priority,
    Project,
}

pub enum MatchFilter<'a> {
    Completed(bool),
    CompletionDate(NaiveDate),
    Context(&'a str),
    CreationDate(NaiveDate),
    DueDate(NaiveDate),
    Priority(char),
    Project(&'a str),
}

/// Adds a task to the list
///
/// The new task will be inserted at the bottom of todo.txt.
/// In case of success, this function returns a tuple containing the task ID and its formatted string.
pub fn add(
    todo: &str,
    priority: Option<char>,
    creation_date: Option<NaiveDate>,
    insert_creation_date: bool,
) -> Result<(usize, String), String> {
    let mut file = OpenOptions::new()
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
    let mut tasks = read_todo_file()?;
    let (position, task_to_complete) = locate_task(id, &tasks)?;

    if task_to_complete.to_ascii_lowercase().starts_with('x') {
        return Err(String::from("This task is already marked as done"));
    }

    tasks.insert_str(position, "x ");
    overwrite_todo_file(&tasks)
}

/// Removes a task from the list
///
/// # Errors
///
/// - couldn't find task with given ID
pub fn remove(id: usize) -> Result<(), String> {
    let tasks = read_todo_file()?;
    let tasks = remove_tasks(vec![id], &tasks)?;
    overwrite_todo_file(&tasks)
}

/// Moves completed tasks to the archive file (done.txt)
///
/// When the operation is completed, the function returns the number of moved tasks.
pub fn archive() -> Result<usize, String> {
    let tasks = read_todo_file()?;
    let completed_tasks = locate_completed_tasks(&tasks);

    let completed_tasks_str =
        completed_tasks
            .iter()
            .fold(String::new(), |mut serialized, (_, task)| {
                serialized.push_str(task);
                serialized.push('\n');
                serialized
            });
    let mut done_file = OpenOptions::new()
        .append(true)
        .open(DONETXT_PATH)
        .map_err(|e| e.to_string())?;
    done_file
        .write(completed_tasks_str.as_bytes())
        .map_err(|e| e.to_string())?;

    let completed_tasks_ids = completed_tasks.iter().map(|(id, _)| *id).collect();
    let filtered_tasks = remove_tasks(completed_tasks_ids, &tasks)?;
    overwrite_todo_file(&filtered_tasks)?;
    Ok(completed_tasks.len())
}

/// Returns the tasks matching the filters
pub fn list(
    matching: &[MatchFilter],
    sort_by: &[SortFilter],
) -> Result<Vec<(usize, String)>, String> {
    let tasks = read_todo_file()?;
    let tasks = enumerate_tasks(&tasks);

    let tasks = filter_tasks(&tasks, matching);
    let tasks = sort_tasks(&tasks, sort_by);
    Ok(tasks)
}

fn read_todo_file() -> Result<String, String> {
    let mut todo_file = File::open(TODOTXT_PATH).map_err(|e| e.to_string())?;
    let mut tasks = String::new();
    todo_file
        .read_to_string(&mut tasks)
        .map_err(|e| e.to_string())?;
    Ok(tasks)
}

fn overwrite_todo_file(tasks: &str) -> Result<(), String> {
    let mut todo_file = File::create(TODOTXT_PATH).map_err(|e| e.to_string())?;
    todo_file
        .write(tasks.as_bytes())
        .map_err(|e| e.to_string())?;
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

fn locate_task(id: usize, tasks: &str) -> Result<(usize, String), String> {
    let mut position = 0;
    let (_, task) = tasks
        .lines()
        .enumerate()
        .find(|(i, line)| {
            let is_task_located = {
                let line_nth = i + 1;
                line_nth == id
            };
            if is_task_located {
                return true;
            }

            position += line.as_bytes().len() + NEWLINE_BYTE;
            false
        })
        .ok_or(String::from("Unable to find the task"))?;
    Ok((position, task.to_string()))
}

fn locate_completed_tasks(tasks: &str) -> Vec<(usize, String)> {
    tasks
        .lines()
        .enumerate()
        .fold(Vec::new(), |mut completed_tasks, (i, task)| {
            if task.to_ascii_lowercase().starts_with('x') {
                completed_tasks.push((i + 1, task.to_string()));
            }
            completed_tasks
        })
}

fn remove_tasks(ids: Vec<usize>, tasks: &str) -> Result<String, String> {
    let nb_tasks = tasks.lines().count();
    if ids.iter().any(|id| *id > nb_tasks) {
        return Err(String::from("Invalid id"));
    }
    let filtered_tasks =
        tasks
            .lines()
            .enumerate()
            .fold(String::new(), |mut filtered, (i, task)| {
                if !ids.iter().any(|id| *id == i + 1) {
                    filtered.push_str(task);
                    filtered.push('\n');
                }
                filtered
            });
    Ok(filtered_tasks)
}

fn filter_tasks(tasks: &[(usize, String)], filters: &[MatchFilter]) -> Vec<(usize, String)> {
    todo!()
}

fn sort_tasks(tasks: &[(usize, String)], filters: &[SortFilter]) -> Vec<(usize, String)> {
    todo!()
}

fn enumerate_tasks(tasks: &str) -> Vec<(usize, String)> {
    tasks
        .lines()
        .enumerate()
        .map(|(i, task)| (i + 1, task.to_string()))
        .collect::<Vec<(usize, String)>>()
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

    #[test]
    fn locate_tasks_correctly() {
        const TASKS: &str = "One\nTwo\nThree\nFour\n";
        assert_eq!(Ok((0, String::from("One"))), locate_task(1, TASKS));
        assert_eq!(Ok((8, String::from("Three"))), locate_task(3, TASKS));
        assert_eq!(Ok((14, String::from("Four"))), locate_task(4, TASKS));
        assert_eq!(
            Err(String::from("Unable to find the task")),
            locate_task(5, TASKS)
        );
        assert_eq!(
            Err(String::from("Unable to find the task")),
            locate_task(6, TASKS)
        );
    }

    #[test]
    fn locate_each_completed_task() {
        assert_eq!(
            vec![
                (1, String::from("x A")),
                (3, String::from("x C")),
                (7, String::from("x"))
            ],
            locate_completed_tasks("x A\nB x\nx C\n(x) D\n x E\nF\nx")
        );
        assert_eq!(
            Vec::<(usize, String)>::new(),
            locate_completed_tasks("A\nB\nC\n")
        );
    }

    #[test]
    fn remove_specified_tasks() {
        const TASKS: &str = "T1\nT2\nT3\n";
        assert_eq!(Ok(String::from("T2\nT3\n")), remove_tasks(vec![1], TASKS));
        assert_eq!(Ok(String::from("T1\nT3\n")), remove_tasks(vec![2], TASKS));
        assert_eq!(Ok(String::from("T1\nT2\n")), remove_tasks(vec![3], TASKS));
        assert_eq!(Ok(String::from("")), remove_tasks(vec![1, 2, 3], TASKS));
        assert_eq!(
            Err(String::from("Invalid id")),
            remove_tasks(vec![4], TASKS)
        );
    }

    #[test]
    fn keep_tasks_matching_single_criteria() {
        let tasks = [
            (1, String::from("x T1")),
            (2, String::from("x 2020-05-02 2020-05-01 T2")),
            (3, String::from("T3 @context")),
            (4, String::from("2020-05-02 T4")),
            (5, String::from("T5 due:2020-05-02")),
            (6, String::from("(A) T6")),
            (7, String::from("T7 +project")),
        ];

        let completed_tasks = filter_tasks(&tasks, &[MatchFilter::Completed(true)]);
        assert_eq!(
            vec![
                (1, String::from("x T1")),
                (2, String::from("x 2020-05-02 2020-05-01 T2"))
            ],
            completed_tasks
        );

        let tasks_with_completed_date = filter_tasks(
            &tasks,
            &[MatchFilter::CompletionDate(
                NaiveDate::parse_from_str("2020-05-02", "%Y-%m-%d").unwrap(),
            )],
        );
        assert_eq!(
            vec![(2, String::from("x 2020-05-02 2020-05-01 T2"))],
            tasks_with_completed_date
        );

        let tasks_with_context = filter_tasks(&tasks, &[MatchFilter::Context("context")]);
        assert_eq!(vec![(3, String::from("T3 @context"))], tasks_with_context);

        let tasks_with_creation_date = filter_tasks(
            &tasks,
            &[MatchFilter::CreationDate(
                NaiveDate::parse_from_str("2020-05-01", "%Y-%m-%d").unwrap(),
            )],
        );
        assert_eq!(
            vec![
                (2, String::from("x 2020-05-02 2020-05-01 T2")),
                (4, String::from("2020-05-02 T4"))
            ],
            tasks_with_creation_date
        );

        let tasks_with_due_date = filter_tasks(
            &tasks,
            &[MatchFilter::DueDate(
                NaiveDate::parse_from_str("2020-05-02", "%Y-%m-%d").unwrap(),
            )],
        );
        assert_eq!(
            vec![(5, String::from("T5 due:2020-05-02"))],
            tasks_with_due_date
        );

        let prioritized_tasks = filter_tasks(&tasks, &[MatchFilter::Priority('A')]);
        assert_eq!(vec![(6, String::from("(A) T6"))], prioritized_tasks);

        let tasks_with_project = filter_tasks(&tasks, &[MatchFilter::Project("project")]);
        assert_eq!(vec![(7, String::from("T7 +project"))], tasks_with_project);
    }

    #[test]
    fn keep_tasks_matching_criterias() {
        let tasks = [(1, String::from("@c1 T1 +p1")), (2, String::from("T2 +p1"))];

        let tasks_matching_c1_and_p1 = filter_tasks(
            &tasks,
            &[MatchFilter::Context("c1"), MatchFilter::Project("p1")],
        );
        assert_eq!(
            vec![(1, String::from("@c1 T1 +p1"))],
            tasks_matching_c1_and_p1
        );
    }

    #[test]
    fn sort_tasks_using_single_filter() {
        todo!()
    }

    #[test]
    fn sort_tasks_using_filters() {
        todo!()
    }

    #[test]
    fn output_numbered_tasks() {
        const TASKS: &str = "1\n2\n3\n";
        let numbered_tasks = enumerate_tasks(TASKS);
        assert_eq!(
            vec![
                (1, String::from("1")),
                (2, String::from("2")),
                (3, String::from("3"))
            ],
            numbered_tasks
        );
    }
}
