use chrono::NaiveDate;
use clap::{crate_version, Arg, ArgMatches, Command, ErrorKind};
use colored::*;
use std::str::FromStr;
use todotxt_lib::{MatchFilter, SortFilter};

const ALPHABETIC_FILTER: &str = "alphabetic";
const COMPLETED_FILTER: &str = "completed";
const COMPLETION_DATE_FILTER: &str = "completion";
const CONTEXT_FILTER: &str = "context";
const CREATION_DATE_FILTER: &str = "creation";
const DUE_DATE_FILTER: &str = "due";
const PRIORITY_FILTER: &str = "priority";
const PROJECT_FILTER: &str = "project";

fn main() -> Result<(), String> {
    let mut cmd = cmd();
    let matches = cmd.get_matches_mut();

    match matches.subcommand() {
        Some(("add", matches)) => add(matches, &mut cmd),
        Some(("do", matches)) => mark_as_done(matches, &mut cmd),
        Some(("rm", matches)) => remove(matches, &mut cmd),
        Some(("archive", _)) => archive(),
        Some(("ls", matches)) => list(matches),
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
                        .help("Description of your task")
                        .required(true),
                )
                .arg(
                    Arg::new("priority")
                        .short('p')
                        .long("priority")
                        .value_name("A-Z")
                        .help("Priority of your task"),
                )
                .arg(
                    Arg::new("creation_date")
                        .short('c')
                        .long("creation-date")
                        .value_name("YYYY-MM-DD")
                        .help("Creation date for this task"),
                )
                .arg(
                    Arg::new("no_creation_date")
                        .long("no-creation-date")
                        .conflicts_with("creation_date")
                        .help("Disable creation date for this task"),
                ),
        )
        .subcommand(
            Command::new("do").about("Mark a task as done").arg(
                Arg::new("task_id")
                    .help("Identifying number for the accomplished task")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("rm").about("Remove a task").arg(
                Arg::new("task_id")
                    .help("Identifying number for the task to remove")
                    .required(true),
            ),
        )
        .subcommand(Command::new("archive").about("Move all completed tasks to done.txt"))
        .subcommand(
            Command::new("ls")
                .about("List tasks from todo.txt")
                .arg(
                    Arg::new("sort_by")
                        .short('s')
                        .long("sort")
                        .value_name("FILTERS")
                        .multiple_values(true)
                        .possible_values(&[
                            ALPHABETIC_FILTER,
                            COMPLETED_FILTER,
                            COMPLETION_DATE_FILTER,
                            CONTEXT_FILTER,
                            CREATION_DATE_FILTER,
                            DUE_DATE_FILTER,
                            PRIORITY_FILTER,
                            PROJECT_FILTER,
                        ])
                        .help("Sorting filters to apply, in order of precedence")
                        .next_line_help(true),
                )
                .arg(
                    Arg::new("completed")
                        .short('x')
                        .long("completed")
                        .conflicts_with("not_completed")
                        .help("Completed tasks only"),
                )
                .arg(
                    Arg::new("not_completed")
                        .short('X')
                        .long("not-completed")
                        .conflicts_with("completed")
                        .help("Unfinished tasks only"),
                )
                .arg(
                    Arg::new("completion_date")
                        .short('C')
                        .long("completion")
                        .value_name("YYYY-MM-DD")
                        .help("Completion date to match"),
                )
                .arg(
                    Arg::new("context")
                        .short('o')
                        .long("context")
                        .value_name("CONTEXT")
                        .help("Context to match"),
                )
                .arg(
                    Arg::new("creation_date")
                        .short('c')
                        .long("creation")
                        .value_name("YYYY-MM-DD")
                        .help("Creation date to match"),
                )
                .arg(
                    Arg::new("due_date")
                        .short('d')
                        .long("due")
                        .value_name("YYYY-MM-DD")
                        .help("Due date to match"),
                )
                .arg(
                    Arg::new("priority")
                        .short('P')
                        .long("priority")
                        .value_name("A-Z")
                        .help("Priority to match"),
                )
                .arg(
                    Arg::new("project")
                        .short('p')
                        .long("project")
                        .value_name("PROJECT")
                        .help("Project to match"),
                ),
        )
}

fn add(matches: &ArgMatches, cmd: &mut Command) -> Result<(), String> {
    let todo = matches.value_of("todo").unwrap();

    let priority = if let Some(matched_value) = matches.value_of("priority") {
        match_alphabetic_char(matched_value).map_or_else(
            |error| cmd.error(ErrorKind::ValueValidation, error).exit(),
            |letter| Some(letter),
        )
    } else {
        None
    };

    let creation_date = if let Some(matched_value) = matches.value_of("creation_date") {
        match_iso8601_date(matched_value).map_or_else(
            |error| cmd.error(ErrorKind::ValueValidation, error).exit(),
            |date| Some(date),
        )
    } else {
        None
    };

    let insert_creation_date = !matches.is_present("no_creation_date");

    let (task_id, task_entry) =
        todotxt_lib::add(todo, priority, creation_date, insert_creation_date)?;
    print_task(task_id, &task_entry);

    Ok(())
}

fn mark_as_done(matches: &ArgMatches, cmd: &mut Command) -> Result<(), String> {
    let id = matches
        .value_of_t("task_id")
        .unwrap_or_else(|error| cmd.error(ErrorKind::ValueValidation, error).exit());
    todotxt_lib::mark_as_done(id)?;
    print_task(id, "marked as done");
    Ok(())
}

fn remove(matches: &ArgMatches, cmd: &mut Command) -> Result<(), String> {
    let id = matches
        .value_of_t("task_id")
        .unwrap_or_else(|error| cmd.error(ErrorKind::ValueValidation, error).exit());
    todotxt_lib::remove(id)?;
    print_task(id, "removed");
    Ok(())
}

fn archive() -> Result<(), String> {
    let nb_archived_tasks = todotxt_lib::archive()?;
    println!("{} task(s) archived", nb_archived_tasks);
    Ok(())
}

fn list(matches: &ArgMatches) -> Result<(), String> {
    let match_filters = {
        let mut filters = Vec::new();

        if matches.is_present("completed") {
            filters.push(MatchFilter::Completed(true));
        } else if matches.is_present("not_completed") {
            filters.push(MatchFilter::Completed(false));
        }

        if let Some(matched_value) = matches.value_of("completion_date") {
            let date = match_iso8601_date(matched_value)?;
            filters.push(MatchFilter::CompletionDate(date));
        }

        if let Some(matched_value) = matches.value_of("creation_date") {
            let date = match_iso8601_date(matched_value)?;
            filters.push(MatchFilter::CreationDate(date));
        }

        if let Some(matched_value) = matches.value_of("due_date") {
            let date = match_iso8601_date(matched_value)?;
            filters.push(MatchFilter::DueDate(date));
        }

        if let Some(context) = matches.value_of("context") {
            filters.push(MatchFilter::Context(context));
        }

        if let Some(matched_value) = matches.value_of("priority") {
            let p = match_alphabetic_char(matched_value)?;
            filters.push(MatchFilter::Priority(p));
        }

        if let Some(project) = matches.value_of("project") {
            filters.push(MatchFilter::Project(project));
        }
        filters
    };

    let sort_filters = {
        let input_filters = matches
            .values_of("sort_by")
            .map_or(Vec::new(), |filters| filters.collect::<Vec<&str>>());

        input_filters
            .iter()
            .map(|filter| match *filter {
                ALPHABETIC_FILTER => SortFilter::Alphabetic,
                COMPLETED_FILTER => SortFilter::Completed,
                COMPLETION_DATE_FILTER => SortFilter::CompletionDate,
                CONTEXT_FILTER => SortFilter::Context,
                CREATION_DATE_FILTER => SortFilter::CreationDate,
                DUE_DATE_FILTER => SortFilter::DueDate,
                PRIORITY_FILTER => SortFilter::Priority,
                PROJECT_FILTER => SortFilter::Project,
                _ => SortFilter::CreationDate,
            })
            .collect::<Vec<SortFilter>>()
    };

    let tasks = todotxt_lib::list(&match_filters, &sort_filters)?;
    tasks.iter().for_each(|(id, task)| print_task(*id, task));
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
