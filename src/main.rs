// make the vim extension go to normal mode when pressing kj
// make cursor different in insert and normal mode (| for insert, box for insert)
//
// use flags to input things:
//      jrl -w to show week notes
//      jrl -m to show months notes highlights

mod cli;
mod question_structs;
use question_structs::Question;
mod entries_information;
mod file_parsing;
mod loops;
mod pager;
mod utility;

use core::f64;
use rand::Rng;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;

use crossterm::{execute, terminal};
use rustyline::error::ReadlineError;
use std::io;

const JRL_DIR_NAME: &str = ".jrl";
const QUESTION_FILE_NAME: &str = "questions.txt";

fn main() -> rustyline::Result<()> {
    let mut jrl_dir_path = home::home_dir().expect("Could not find home directory");
    jrl_dir_path.push(JRL_DIR_NAME);

    if cli::show_entries() {
        match loops::view_files(&jrl_dir_path) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                let mut stdout = io::stdout();
                execute!(stdout, terminal::LeaveAlternateScreen)?;
                terminal::disable_raw_mode()?;

                return Err(ReadlineError::Io(io::Error::other(e.to_string())));
            }
        }
    }

    if let Some(entries_to_consider) = cli::show_analytics() {
        entries_information::get_statistics(jrl_dir_path, entries_to_consider)?;

        return Ok(());
    }

    let days_before_today: i64 = cli::parse_days_before();

    let today_file = utility::get_day_file_name(days_before_today);
    let today_file_path = jrl_dir_path.join(&today_file);
    let questions_file_path = jrl_dir_path.join(QUESTION_FILE_NAME);

    let mut write_question_gap = true;

    if !file_parsing::exists_today_file(&jrl_dir_path, &today_file)? {
        fs::write(&today_file_path, "")?;
        write_question_gap = false;
    }
    if fs::metadata(&today_file_path)?.len() == 0 {
        write_question_gap = false;
    }

    // copy current dir questions file in ~/.jrl folder or create an empty one otherwise if not existing there
    let mut this_dir_questions_path = PathBuf::new();
    this_dir_questions_path.push("./");
    this_dir_questions_path.push(QUESTION_FILE_NAME);

    let tmp_file = file_parsing::generate_jumbled_questions_file_name();

    if cli::install_questions() {
        if this_dir_questions_path.exists() {
            fs::copy(&this_dir_questions_path, &questions_file_path)?;
            println!("Succesfully installed the questions.txt file");

            if tmp_file.exists() {
                std::fs::remove_file(tmp_file)?;
            }

            return Ok(());
        } else {
            println!("No quesions.txt file in the current directory");
            return Ok(());
        }
    }
    if !questions_file_path.exists() {
        if this_dir_questions_path.exists() {
            fs::copy(&this_dir_questions_path, &questions_file_path)?;
        } else {
            fs::write(
                &questions_file_path,
                "l: Long question\ns: Short question\n",
            )
            .expect("Failed to create question file\n");
        }

        if tmp_file.exists() {
            std::fs::remove_file(tmp_file)?;
        }
    }

    let mut file: fs::File = OpenOptions::new().append(true).open(&today_file_path)?;

    let mut question_to_ask: Question = Question::default();
    let mut question_chance: f64 = 1.0;

    if cli::parse_args(
        &mut question_to_ask,
        &mut file,
        &mut question_chance,
        &write_question_gap,
    )? {
        return Ok(());
    }

    let mut rng = rand::rng();

    if question_to_ask == Question::default() && rng.random::<f64>() < question_chance {
        question_to_ask = file_parsing::get_question(&questions_file_path)?;
    }

    if question_to_ask == Question::default() {
        return Ok(());
    }

    loops::get_input(question_to_ask, &mut file, write_question_gap)?;

    Ok(())
}
