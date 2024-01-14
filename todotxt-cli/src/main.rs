use chrono::NaiveDate;
use clap::{crate_version, value_parser, Arg, ArgAction, ArgMatches, Command, ErrorKind};
use colored::*;
use std::str::FromStr;
use todotxt_lib;

fn main() -> Result<(), String> {
    let mut cmd = cmd();
    let matches = cmd.get_matches_mut();

    match matches.subcommand() {
        Some(("add", matches)) => add(matches, &mut cmd),
        Some(("do", matches)) => mark_as_done(matches),
        Some(("rm", matches)) => remove(matches),
        Some(("archive", _)) => archive(),
        _ => Ok(()),
    }
}

fn cmd() -> Command<'static> {
    Command::new("todotxt-rs")
        .version(crate_version!())
        .author("Florent Nuttens")
        .about("Ask more of your todo.txt file")
        .subcommand(
            Command::new("add")
                .about("Add a new task to todo.txt")
                .arg(
                    Arg::new("todo")
                        .action(ArgAction::Set)
                        .help("Description of your task")
                        .required(true),
                )
                .arg(
                    Arg::new("priority")
                        .action(ArgAction::Set)
                        .short('p')
                        .long("priority")
                        .value_name("A-Z")
                        .help("Priority of your task"),
                )
                .arg(
                    Arg::new("creation_date")
                        .action(ArgAction::Set)
                        .short('c')
                        .long("creation-date")
                        .value_name("YYYY-MM-DD")
                        .help("Creation date for this task"),
                )
                .arg(
                    Arg::new("no_creation_date")
                        .action(ArgAction::SetTrue)
                        .long("no-creation-date")
                        .conflicts_with("creation_date")
                        .help("Disable creation date for this task"),
                ),
        )
        .subcommand(
            Command::new("do").about("Mark a task as done").arg(
                Arg::new("task-id")
                    .action(ArgAction::Set)
                    .help("Identifying number for the accomplished task")
                    .value_parser(value_parser!(usize))
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("rm").about("Remove a task").arg(
                Arg::new("task-id")
                    .action(ArgAction::Set)
                    .help("Identifying number for the task to remove")
                    .value_parser(value_parser!(usize))
                    .required(true),
            ),
        )
        .subcommand(Command::new("archive").about("Move all completed tasks to done.txt"))
}

fn add(matches: &ArgMatches, cmd: &mut Command) -> Result<(), String> {
    let todo: &String = matches.get_one("todo").expect("Todo should not be empty");

    let priority = if let Some(matched_value) = matches.get_one::<String>("priority") {
        match_alphabetic_char(matched_value).map_or_else(
            |error| cmd.error(ErrorKind::ValueValidation, error).exit(),
            |letter| Some(letter),
        )
    } else {
        None
    };

    let creation_date = if let Some(matched_value) = matches.get_one::<String>("creation_date") {
        match_iso8601_date(matched_value).map_or_else(
            |error| cmd.error(ErrorKind::ValueValidation, error).exit(),
            |date| Some(date),
        )
    } else {
        None
    };

    let insert_creation_date: bool = !matches
        .get_one::<bool>("no_creation_date")
        .copied()
        .unwrap_or_default();

    let (task_id, task_entry) =
        todotxt_lib::add(todo, priority, creation_date, insert_creation_date)?;
    print_task(task_id, &task_entry);

    Ok(())
}

fn mark_as_done(matches: &ArgMatches) -> Result<(), String> {
    let id = *matches.get_one::<usize>("task-id").unwrap();
    todotxt_lib::mark_as_done(id)?;
    print_task(id, "marked as done");
    Ok(())
}

fn remove(matches: &ArgMatches) -> Result<(), String> {
    let id = *matches.get_one::<usize>("task-id").unwrap();
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
    fn verify_app() {
        cmd().debug_assert();
    }

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
