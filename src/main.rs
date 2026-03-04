// make the vim extension go to normal mode when pressing kj
// make cursor different in insert and normal mode (| for insert, box for insert)
//
// use flags to input things:
//      jrl -w to show week notes
//      jrl -m to show months notes highlights

mod cli;
mod question_structs;
use question_structs::Question;
mod file_parsing;
mod loops;
mod pager;
mod utility;

use home;
use core::f64;
use std::path::PathBuf;
use rand::Rng;
use std::fs;
use std::fs::OpenOptions;

use crate::{cli::parse_days_before, loops::get_jrl_files, question_structs::{QuestionType, Informative, ChunkParser, PromptQuestionType}};

const JRL_DIR_NAME: &str = ".jrl";
const QUESTION_FILE_NAME: &str = "questions.txt";

use crossterm::{execute, terminal};
use rustyline::error::ReadlineError;
use std::io;

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

                return Err(ReadlineError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    e.to_string(),
                )));
            }
        }
    }

    if cli::show_analytics() {
        let entries_to_consider = 30;

        let all_files = get_jrl_files(&jrl_dir_path)?;
        let recent_entries = &all_files[all_files.len().saturating_sub(entries_to_consider)..];

        let mut best_rating: f64 = f64::MIN;
        let mut best_rating_day: String = "".to_string();
        
        let mut worst_rating: f64 = f64::MAX;
        let mut worst_rating_day: String = "".to_string();

        let mut best_description: String = "".to_string();
        let mut worst_description: String = "".to_string();

        let mut average_rating: f64 = 0.0;
        let mut files_rated: f64 = 0.0;

        let mut entries_number = 0;

        for file in recent_entries {
            let  file_content = fs::read_to_string(file)?;
            entries_number += 1;
            let mut lines = file_content.lines().peekable();

            let mut description: String = "".to_string();
            let mut final_file_rating: Option<f64> = None;

            loop {
                match pager::get_next_chunk(&mut lines) {
                    Ok(this_chunk) => {
                        let chunk_type = this_chunk.get_type().unwrap_or_else(|_| { QuestionType::Empty});
                        let prompt_type = this_chunk.get_prompt_type()?;
                        let mut answer_iter = this_chunk.get_answer()?.into_iter();
                        let info = this_chunk.get_informative()?.get_text()?;

                        if prompt_type == PromptQuestionType::Rating {

                            match this_chunk.get_answer()?[0].get_text()?.as_str().parse::<f64>() {
                                Ok(rating) => {
                                    final_file_rating = Some(rating);
                                }
                                Err(_) => {}
                            }
                        }
                        else if prompt_type == PromptQuestionType::Description {
                            if let Some(question) = answer_iter.next() {
                                description += format!("    [{}] {}", info, question.get_text()?).as_str();
                            }

                            while let Some(question) = answer_iter.next() {
                                description += format!("{}", question.get_text()?).as_str();
                            }
                        } 
                        
                    }
                    Err(pager::ChunkError::UnexpectedFileEnd) => {
                        break;
                    }
                    Err(_) => { }
                }
            }


            if let Some(file_rating) = final_file_rating{
                files_rated += 1.0;
                average_rating += file_rating;
                
                if file_rating > best_rating{
                    best_rating = file_rating;
                    if let Some(file_name) = file.file_stem(){
                        best_rating_day = file_name.to_string_lossy().to_string();
                        best_rating_day = best_rating_day.replace("-", ".");
                        best_description = description.clone();
                    }
                }
                if file_rating < worst_rating{
                    worst_rating = file_rating;
                    if let Some(file_name) = file.file_stem() {
                        worst_rating_day = file_name.to_string_lossy().to_string();
                        worst_rating_day = worst_rating_day.replace("-", ".");
                        worst_description = description.clone();
                    }
                }
            }
        }
        println!("Analytics for the past {} entries:\n", entries_number);

        if best_rating == f64::MIN {
            println!("No ratings given for the past {} days.", entries_number);
        }
        else if best_rating == worst_rating {
            println!("{} was your best and worst day with {} rating:", best_rating_day, best_rating);
            println!("{}", best_description);
        }
        else {
            let average_rating = average_rating / files_rated;
            println!("Average day rating: {:.2}\n", average_rating);

            println!("{} was your best day with {} rating:", best_rating_day, best_rating);
            println!("{}", best_description);

            println!("\n{} was your worst day with {} rating:", worst_rating_day, worst_rating);
            println!("{}", worst_description);
        }

        return Ok(());
    }
    
    let days_before_today: i64 = parse_days_before();

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
    this_dir_questions_path.push(   QUESTION_FILE_NAME);

    let tmp_file = file_parsing::generate_jumbled_questions_file_name();

    if cli::install_questions() {
        if this_dir_questions_path.exists(){ 
            fs::copy(&this_dir_questions_path, &questions_file_path)?;
            println!("Succesfully installed the questions.txt file");

            if tmp_file.exists() {
                std::fs::remove_file(tmp_file)?;
            }
 
            return Ok(());
        }
        else {
            println!("No quesions.txt file in the current directory");
            return Ok(());
        }
    }
    if !questions_file_path.exists() {
        if this_dir_questions_path.exists(){ 
            fs::copy(&this_dir_questions_path, &questions_file_path)?;
        }
        else {
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

    let mut file: fs::File = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&today_file_path)?;

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
