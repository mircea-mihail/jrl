use crate::file_parsing;
use crate::question_structs::Question;

use clap::Parser;
use std::io::Write;

pub const DESCRIPTION_QUESTION_STR: &str = "l: Add a description about the day:";
pub const NOTE_QUESTION_STR: &str = "l: Add a note during the day:";
pub const RATING_QUESTION_STR: &str = "s: Add a rating for your day out of ten:";

const DEFAULT_DESCRIPTION: &str = "No description provided";
const DEFAULT_NOTE: &str = "No note provided";
const DEFAULT_RATING: &str = "1212.1212";
const DEFAULT_SOMETIMES: &str = "true";
const DEFAULT_ENTRIES: &str = "0000-00-00";
const DEFAULT_UPDATE_QUESTIONS: &str = "true";
const DEFAULT_ANALYTICS: &str = "30";

const QUESTION_CHANCE: f64 = 0.5;

#[derive(Parser)]
pub struct Cli {
    /// Talk about how your day was
    #[arg(
        short,
        long,
        num_args = 0..=1,
        default_missing_value = DEFAULT_DESCRIPTION
    )]
    description: Option<String>,

    /// Add a short note during the day
    #[arg(
        short,
        long,
        num_args = 0..=1,
        default_missing_value = DEFAULT_NOTE
    )]
    note: Option<String>,

    /// Rate your day out of 10 (can be any number)
    #[arg(
        short,
        long,
        num_args = 0..=1,
        default_missing_value = DEFAULT_RATING
    )]
    rating: Option<f64>,

    /// Lower chances of a question being asked
    #[arg (
        short,
        long,
        num_args = 0..=1,
        default_missing_value = DEFAULT_SOMETIMES
    )]
    sometimes: Option<bool>,

    /// Show all entries into the journal. Argument format: yyyy-mm-dd
    #[arg (
        short,
        long,
        num_args = 0..=1,
        default_missing_value = DEFAULT_ENTRIES
    )]
    entries: Option<String>,

    /// Update journal from x days ago
    #[arg(short, long, num_args = 1)]
    update: Option<i64>,

    /// Use the questions from the questions.txt file in the current directory
    #[arg (
        long,
        short,
        num_args = 0..=1,
        default_missing_value = DEFAULT_UPDATE_QUESTIONS
    )]
    install_questions: Option<bool>,

    /// Show analytics for the past x journal entries. 30 by default.
    #[arg (
        short,
        long,
        num_args = 0..=1,
        default_missing_value = DEFAULT_ANALYTICS
    )]
    analytics: Option<usize>,
}

pub fn install_questions() -> bool {
    let args = Cli::parse();

    let mut install_questions: bool = false;
    if let Some(s) = args.install_questions {
        install_questions = s;
    }

    install_questions
}

pub fn show_analytics() -> Option<usize> {
    let args = Cli::parse();

    args.analytics
}

pub fn parse_days_before() -> i64 {
    let args = Cli::parse();

    let mut days_before_today: i64 = 0;
    if let Some(s) = args.update {
        days_before_today = s;

        println!(
            "Update entry for {}:",
            (chrono::offset::Local::now() - chrono::Duration::days(days_before_today))
                .format("%d %b %Y")
        );
    }

    days_before_today
}

pub fn show_entries() -> Option<String> {
    let args = Cli::parse();
    let today = chrono::offset::Local::now().format("%Y-%m-%d").to_string();

    if let Some(e) = args.entries {
        if e != DEFAULT_ENTRIES {
            return Some(e)
        }
        else {
            return Some(today)
        }
    }

    None
}

pub fn parse_args(
    question: &mut Question,
    file: &mut std::fs::File,
    question_chance: &mut f64,
    write_question_gap: &bool,
) -> std::io::Result<bool> {
    let args = Cli::parse();

    if let Some(a) = args.description {
        *question = DESCRIPTION_QUESTION_STR.to_string().into();

        if a != DEFAULT_DESCRIPTION {
            if *write_question_gap {
                file.write_all("\n".as_bytes())?;
            }
            file_parsing::write_question(file, question)?;
            file_parsing::write_answer(file, &a)?;
            return Ok(true);
        }
    }
    if let Some(a) = args.note {
        *question = NOTE_QUESTION_STR.to_string().into();

        if a != DEFAULT_NOTE {
            if *write_question_gap {
                file.write_all("\n".as_bytes())?;
            }
            file_parsing::write_question(file, question)?;
            file_parsing::write_answer(file, &a)?;
            return Ok(true);
        }
    }
    if let Some(a) = args.rating {
        *question = RATING_QUESTION_STR.to_string().into();

        if a != DEFAULT_RATING.parse::<f64>().unwrap() {
            if *write_question_gap {
                file.write_all("\n".as_bytes())?;
            }
            file_parsing::write_question(file, question)?;
            file_parsing::write_answer(file, &a.to_string())?;
            return Ok(true);
        }
    }
    if let Some(s) = args.sometimes
        && s == DEFAULT_SOMETIMES.parse::<bool>().unwrap()
    {
        *question_chance = QUESTION_CHANCE;
    }

    Ok(false)
}
