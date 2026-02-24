use crate::file_parsing;
use crate::pager;
use crate::question_structs::{Informative, Question, QuestionType};

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
                if line == "".to_string() {
                    break;
                }
                if !wrote_quesiton {
                    if write_question_gap {
                        file.write_all("\n".as_bytes())?;
                    }

                    file_parsing::write_question(&file, &question)?;
                    wrote_quesiton = true;
                }

                file_parsing::write_answer(&file, &line)?;

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

pub fn get_jrl_files(jrl_dir_path: &std::path::PathBuf) ->  io::Result<Vec<std::path::PathBuf>>{
    let dir_files = fs::read_dir(jrl_dir_path)?;
    let mut journal_paths: Vec<std::path::PathBuf> = Vec::new();

    for file in dir_files {
        let path = file?.path();

        if let Some(stem) = path.file_stem() {
            let string_split_stem = stem.to_string_lossy();
            let stem_vec: Vec<&str> = string_split_stem.split("-").collect();
            if stem_vec.len() == 3
                && stem_vec[0].len() == 4
                && stem_vec[1].len() == 2
                && stem_vec[2].len() == 2
            {
                journal_paths.push(path);
            }
        }
    }
    journal_paths.sort();

    Ok(journal_paths)
}

pub fn view_files(jrl_dir_path: &std::path::PathBuf) -> io::Result<()> {
    let journal_paths = get_jrl_files(jrl_dir_path)?;
    let idx_max_len = journal_paths.len() - 1;

    let mut file_index = idx_max_len;
    let mut height_index = 0;

    let mut height_changed = false;
    let mut file_changed = false;

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen)?;

    let mut file_content = fs::read_to_string(&journal_paths[file_index])?;
    let mut formatted_content =
        pager::format_content(&file_content).unwrap_or(ERROR_PARSING_FILE.to_string());
    let mut terminal_lines = pager::parse_display_text(&formatted_content)?;

    pager::write_display_content(
        &journal_paths[file_index],
        height_index,
        &terminal_lines,
        &stdout,
    )?;

    loop {
        let event = event::read()?;
        if let Event::Resize(_, _) = event {
            terminal_lines = pager::parse_display_text(&formatted_content)?;

            pager::write_display_content(
                &journal_paths[file_index],
                height_index,
                &terminal_lines,
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
                KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter=> {
                    let (_, term_height) = terminal::size()?;
                    if terminal_lines.len() - height_index >= term_height as usize {
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
                    if terminal_lines.len() > term_height as usize  {
                        let desired_height = terminal_lines.len() + 1 - term_height as usize;
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
                &terminal_lines,
                &stdout,
            )?;
        }

        if file_changed {
            file_changed = false;
            file_index = file_index % (idx_max_len + 1);
            height_index = 0;

            file_content = fs::read_to_string(&journal_paths[file_index])?;
            formatted_content =
                pager::format_content(&file_content).unwrap_or(ERROR_PARSING_FILE.to_string());
            terminal_lines = pager::parse_display_text(&formatted_content)?;

            pager::write_display_content(
                &journal_paths[file_index],
                height_index,
                &terminal_lines,
                &stdout,
            )?;
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    stdout.flush()?;
    return Ok(());
}
