use crate::file_parsing;
use crate::pager;
use crate::question_structs::{Informative, Question, QuestionType};
use crate::utility;

use std::io::{self, Write};

use std::fs;

use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor, EditMode};

use crossterm::{
    event::{self, Event, KeyCode},
    execute, terminal,
};

const ERROR_PARSING_FILE: &str = "[ error encountered parsing the file ]";

pub fn get_input(
    question: Question,
    file: &mut fs::File,
    write_question_gap: bool,
) -> rustyline::Result<()> {
    println!("{}", question.get_text()?);

    let config = Config::builder().edit_mode(EditMode::Vi).build();
    let mut rl = DefaultEditor::with_config(config)?;

    let mut wrote_quesiton = false;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    break;
                }
                if !wrote_quesiton {
                    if write_question_gap {
                        file.write_all("\n".as_bytes())?;
                    }

                    file_parsing::write_question(file, &question)?;
                    wrote_quesiton = true;
                }

                file_parsing::write_answer(file, &line)?;

                if question.get_type()? == QuestionType::Short {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

pub fn view_files(jrl_dir_path: &std::path::PathBuf, day_to_show: &str) -> io::Result<()> {
    let journal_paths = utility::get_jrl_files(jrl_dir_path)?;
    let idx_max_len = journal_paths.len() - 1;

    let mut file_index = idx_max_len;
    let mut height_index = 0;

    let mut height_changed = false;
    let mut file_changed = false;

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen)?;

    if let Some(input_idx) = journal_paths
        .iter()
        .position(|p| p.file_stem().map_or(false, |stem| stem == day_to_show))
    {
        file_index = input_idx;
    }

    let mut file_content = fs::read_to_string(&journal_paths[file_index])?;
    let mut parsed_content =
        pager::parse_date_file(&file_content).unwrap_or(ERROR_PARSING_FILE.to_string());
    let mut formatted_content = pager::format_content(&parsed_content)?;

    pager::write_display_content(
        &journal_paths[file_index],
        height_index,
        &formatted_content,
        &stdout,
    )?;

    loop {
        let event = event::read()?;
        if let Event::Resize(_, _) = event {
            formatted_content = pager::format_content(&parsed_content)?;

            pager::write_display_content(
                &journal_paths[file_index],
                height_index,
                &formatted_content,
                &stdout,
            )?;
        }
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('l') | KeyCode::Right => {
                    file_index += 1;
                    file_changed = true;
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if file_index == 0 {
                        file_index = idx_max_len
                    } else {
                        file_index -= 1;
                    }
                    file_changed = true;
                }
                KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => {
                    let (_, term_height) = terminal::size()?;
                    if formatted_content.len() - height_index >= term_height as usize {
                        height_index += 1;
                        height_changed = true;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up | KeyCode::Backspace => {
                    if height_index != 0 {
                        height_index -= 1;
                        height_changed = true;
                    }
                }
                KeyCode::Char('G') => {
                    let (_, term_height) = terminal::size()?;
                    if formatted_content.len() > term_height as usize {
                        let desired_height = formatted_content.len() + 1 - term_height as usize;
                        if height_index != desired_height {
                            height_index = desired_height;
                            height_changed = true;
                        }
                    }
                }
                KeyCode::Char('g') => {
                    if height_index != 0 {
                        height_index = 0;
                        height_changed = true;
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => break,
                _ => {}
            }
        }
        if height_changed {
            height_changed = false;
            pager::write_display_content(
                &journal_paths[file_index],
                height_index,
                &formatted_content,
                &stdout,
            )?;
        }

        if file_changed {
            file_changed = false;
            file_index %= idx_max_len + 1;
            height_index = 0;

            file_content = fs::read_to_string(&journal_paths[file_index])?;
            parsed_content =
                pager::parse_date_file(&file_content).unwrap_or(ERROR_PARSING_FILE.to_string());
            formatted_content = pager::format_content(&parsed_content)?;

            pager::write_display_content(
                &journal_paths[file_index],
                height_index,
                &formatted_content,
                &stdout,
            )?;
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    stdout.flush()?;

    Ok(())
}
