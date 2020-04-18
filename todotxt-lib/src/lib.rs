mod config;

use chrono::NaiveDate;

pub fn add(
    todo: &str,
    priority: Option<char>,
    creation_date: Option<NaiveDate>,
    insert_creation_date: bool,
) -> Result<(), String> {
    todo!();
}

#[cfg(test)]
mod tests {}
