use chrono::NaiveDate;
use clap::{crate_version, App, Arg, ArgMatches};
use colored::*;
use std::str::FromStr;
use todotxt_lib;

fn main() -> Result<(), String> {
    let matches = App::new("todotxt-rs")
        .version(crate_version!())
        .author("Florent Nuttens")
        .about("Ask more of your todo.txt file")
        .subcommand(
            App::new("add")
                .about("Add a new task to todo.txt")
                .arg(
                    Arg::with_name("todo")
                        .help("Description of your task")
                        .required(true),
                )
                .arg(
                    Arg::with_name("priority")
                        .short('p')
                        .long("priority")
                        .value_name("A-Z")
                        .help("Priority of your task"),
                )
                .arg(
                    Arg::with_name("creation_date")
                        .short('c')
                        .long("creation-date")
                        .value_name("YYYY-MM-DD")
                        .help("Creation date for this task"),
                )
                .arg(
                    Arg::with_name("no_creation_date")
                        .long("no-creation-date")
                        .conflicts_with("creation_date")
                        .help("Disable creation date for this task"),
                ),
        )
        .subcommand(
            App::new("do").about("Mark a task as done").arg(
                Arg::with_name("task-id")
                    .help("Identifying number for the accomplished task")
                    .required(true),
            ),
        )
        .subcommand(
            App::new("rm").about("Remove a task").arg(
                Arg::with_name("task-id")
                    .help("Identifying number for the task to remove")
                    .required(true),
            ),
        )
        .subcommand(App::new("archive").about("Move all completed tasks to done.txt"))
        .get_matches();

    match matches.subcommand() {
        ("add", Some(matches)) => add(matches),
        ("do", Some(matches)) => mark_as_done(matches),
        ("rm", Some(matches)) => remove(matches),
        ("archive", Some(_)) => archive(),
        _ => Ok(()),
    }
}

fn add(matches: &ArgMatches) -> Result<(), String> {
    let todo = matches.value_of("todo").unwrap();
    let priority = {
        if let Some(matched_value) = matches.value_of("priority") {
            Some(match_alphabetic_char(matched_value)?)
        } else {
            None
        }
    };
    let creation_date = {
        if let Some(matched_value) = matches.value_of("creation_date") {
            Some(match_iso8601_date(matched_value)?)
        } else {
            None
        }
    };
    let insert_creation_date = !matches.is_present("no_creation_date");

    let (task_id, task_entry) =
        todotxt_lib::add(todo, priority, creation_date, insert_creation_date)?;
    print_task(task_id, &task_entry);

    Ok(())
}

fn mark_as_done(matches: &ArgMatches) -> Result<(), String> {
    let id: usize = matches.value_of_t("task-id").unwrap_or_else(|e| e.exit());
    todotxt_lib::mark_as_done(id)?;
    print_task(id, "marked as done");
    Ok(())
}

fn remove(matches: &ArgMatches) -> Result<(), String> {
    let id: usize = matches.value_of_t("task-id").unwrap_or_else(|e| e.exit());
    todotxt_lib::remove(id)?;
    print_task(id, "removed");
    Ok(())
}

fn archive() -> Result<(), String> {
    let nb_archived_tasks = todotxt_lib::archive()?;
    println!("{} task(s) archived", nb_archived_tasks);
    Ok(())
}

fn match_alphabetic_char(value: &str) -> Result<char, &str> {
    const ERROR_MESSAGE: &str =
        "Priority must be a single character between A and Z (case insensitive)";

    char::from_str(value)
        .map_err(|_| ERROR_MESSAGE)
        .and_then(|c| {
            if c.is_ascii_alphabetic() {
                Ok(c.to_ascii_uppercase())
            } else {
                Err(ERROR_MESSAGE)
            }
        })
}

fn match_iso8601_date(value: &str) -> Result<NaiveDate, &str> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "Date must have YYYY-MM-DD format")
}

fn print_task(id: usize, message: &str) {
    let id = format!("{}:", id);
    println!("{} {}", id.yellow().bold(), message);
}

#[cfg(test)]
mod should {
    use super::*;

    #[test]
    fn accept_valid_priority() {
        assert!(match_alphabetic_char("o").is_ok());
        assert!(match_alphabetic_char("K").is_ok());
    }

    #[test]
    fn reject_invalid_priority() {
        assert!(match_alphabetic_char("Ko").is_err());
        assert!(match_alphabetic_char("!").is_err());
        assert!(match_alphabetic_char("3").is_err());
    }

    #[test]
    fn accept_valid_date() {
        assert!(match_iso8601_date("2020-05-02").is_ok());
    }

    #[test]
    fn reject_invalid_date() {
        assert!(match_iso8601_date("rubbish").is_err());
        assert!(match_iso8601_date("02/05/2020").is_err());
        assert!(match_iso8601_date("2020-05-32").is_err());
    }
}
