use std::{collections::BTreeMap, io::Write};

use chrono::Datelike;
use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal,
};

use crate::question_structs::{
    ChunkParser, Informative, PromptQuestionType, QuestionChunk, QuestionType,
};

use rand::Rng;

#[derive(Debug)]
pub enum ChunkError {
    EmptyChunk,
    MalformedChunk,
    UnexpectedFileEnd,
}

pub enum QuoteError {
    EmptyQuote
}

pub fn get_next_chunk(
    lines: &mut std::iter::Peekable<std::str::Lines<'_>>,
) -> Result<QuestionChunk, ChunkError> {
    let mut this_chunk_str: String = "".to_string();

    while let Some(line) = lines.next() {
        if !line.is_empty() {
            this_chunk_str += line;
            this_chunk_str += "\n";
        } else if this_chunk_str.trim().is_empty() {
            return Err(ChunkError::EmptyChunk);
        }
        if lines.peek().is_none() || line.is_empty() {
            this_chunk_str = this_chunk_str.trim().to_string();
            let this_chunk = QuestionChunk::from(this_chunk_str.clone());

            // todo replace get_type with a check_chunk
            match this_chunk.get_type() {
                Ok(_) => return Ok(this_chunk),
                _ => return Err(ChunkError::MalformedChunk),
            }
        }
    }
    Err(ChunkError::UnexpectedFileEnd)
}

pub fn get_quote_from_str(content: &str) -> Result<String, QuoteError> {
    let mut lines: std::iter::Peekable<std::str::Lines<'_>> = content.lines().peekable();
    let mut user_content: String = "".to_string();

    loop {
        match get_next_chunk(&mut lines) {
            Ok(this_chunk) => {
                let mut answer_iter = match this_chunk.get_answer() {
                    Ok(v) => v.into_iter(),
                    Err(_) => continue,
                };
                let prompt_type = match this_chunk.get_prompt_type() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let question_type = match this_chunk.get_type() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if question_type != QuestionType::Empty &&
                    prompt_type != PromptQuestionType::Rating &&
                    question_type != QuestionType::Short 
                {
                    for question in answer_iter.by_ref() {
                        let text = match question.get_text() {
                            Ok(t) => t,
                            Err(_) => continue,
                        };
                        user_content.push_str(&format!(".{}", text));
                    }
                }
            }
            Err(ChunkError::MalformedChunk) => {
            }
            Err(ChunkError::EmptyChunk) => {
            }
            Err(ChunkError::UnexpectedFileEnd) => {
                break;
            }
        }
    }

    let content_phrases: Vec<&str> = user_content
        .split(&['.', '\n', '\t', '!', '?'][..])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if content_phrases.is_empty() {
        return Err(QuoteError::EmptyQuote);
    }

    let mut rng= rand::rng();
    let rand_idx = rng.random_range(0..content_phrases.len());

    Ok(content_phrases[rand_idx].to_string())
}

pub fn parse_date_file(content: &str) -> std::io::Result<String> {
    let mut invalid_chunks_number = 0;
    let mut notes: Vec<String> = Vec::new();
    let mut descriptions: Vec<String> = Vec::new();
    let mut ratings: Vec<String> = Vec::new();
    let mut long_questions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut short_questions: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let mut lines: std::iter::Peekable<std::str::Lines<'_>> = content.lines().peekable();

    loop {
        match get_next_chunk(&mut lines) {
            Ok(this_chunk) => {
                let chunk_type = this_chunk.get_type().unwrap_or(QuestionType::Empty);

                let mut answer_iter = this_chunk.get_answer()?.into_iter();
                let info = this_chunk.get_informative()?.get_text()?;
                let prompt_type = this_chunk.get_prompt_type()?;

                if prompt_type == PromptQuestionType::Rating {
                    for question in answer_iter.by_ref() {
                        ratings.push(format!("[{}] {}", info, question.get_text()?));
                    }
                } else if prompt_type == PromptQuestionType::Description {
                    if let Some(question) = answer_iter.next() {
                        descriptions.push(format!("    [{}] {}", info, question.get_text()?));
                    }

                    for question in answer_iter.by_ref() {
                        descriptions.push(format!("  {}", question.get_text()?));
                    }
                } else if prompt_type == PromptQuestionType::Note {
                    if let Some(question) = answer_iter.next() {
                        notes.push(format!("    [{}] {}", info, question.get_text()?));
                    }

                    for question in answer_iter.by_ref() {
                        notes.push(format!("  {}", question.get_text()?));
                    }
                } else if chunk_type == QuestionType::Long {
                    let question_text = this_chunk.get_question()?.get_text()?;
                    let mut answer: String = "".to_string();

                    if let Some(question) = answer_iter.next() {
                        let answer_line = format!("    [{}] {}", info, question.get_text()?);
                        answer += answer_line.as_str();
                    }

                    for question in answer_iter.by_ref() {
                        let answer_line = format!("\n  {}", question.get_text()?);
                        answer += answer_line.as_str();
                    }

                    long_questions
                        .entry(question_text)
                        .or_default()
                        .push(answer);
                } else {
                    for question in answer_iter.by_ref() {
                        let question_text = this_chunk.get_question()?.get_text()?;
                        short_questions
                            .entry(question_text)
                            .or_default()
                            .push(format!("[{}] {}", info, question.get_text()?));
                    }
                }
            }
            Err(ChunkError::MalformedChunk) => {
                invalid_chunks_number += 1;
            }
            Err(ChunkError::EmptyChunk) => {
                invalid_chunks_number += 1;
            }
            Err(ChunkError::UnexpectedFileEnd) => {
                break;
            }
        }
    }
    let mut return_content = "".to_string();

    if !ratings.is_empty() {
        return_content += "rating: ";
        return_content += ratings.join("->").as_str();
        return_content += "\n\n";
    }

    if !descriptions.is_empty() {
        return_content += "description: \n";
        return_content += descriptions.join("\n").as_str();
        return_content += "\n\n";
    }

    if !notes.is_empty() {
        return_content += "notes: \n";
        return_content += notes.join("\n").as_str();
        return_content += "\n\n";
    }

    if !long_questions.is_empty() {
        return_content += "daily questions: \n";
        for (key, val) in long_questions {
            return_content += key.as_str();
            return_content += "\n";
            return_content += val.join("\n").as_str();
            return_content += "\n\n";
        }
    }

    if !short_questions.is_empty() {
        for (key, val) in short_questions {
            return_content += key.as_str();
            return_content += "\n    ";
            return_content += val.join(" -> ").as_str();
            return_content += "\n\n";
        }
    }

    if invalid_chunks_number != 0 {
        return_content += format!(
            "Encountered {} invalid chunk(s) in file!",
            invalid_chunks_number
        )
        .as_str();
    }

    Ok(return_content)
}

pub fn format_content(content: &str) -> std::io::Result<Vec<String>> {
    let (term_width, _) = terminal::size()?;

    let mut terminal_lines: Vec<String> = Vec::new();
    let mut terminal_line: String = "".to_string();
    let mut line_x;

    for line in content.lines() {
        line_x = 0;

        for mut word in line.split(" ") {
            let mut word_length = word.len();

            if word_length > term_width as usize {
                word = &word[..(term_width) as usize];
                word_length = word.len();
            }

            if word_length + line_x + 1 > term_width as usize {
                terminal_lines.push(terminal_line.clone());
                terminal_line = "".to_string();

                line_x = 0;
            }

            terminal_line += word;
            terminal_line += " ";

            line_x += word_length + 1;
        }
        terminal_lines.push(terminal_line.clone());
        terminal_line = "".to_string();
    }

    terminal_lines.push(terminal_line.clone());
    Ok(terminal_lines)
}

pub fn write_display_content(
    path: &std::path::Path,
    height_index: usize,
    terminal_lines: &[String],
    mut stdout: &std::io::Stdout,
) -> std::io::Result<()> {
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    let (_, term_height) = terminal::size()?;
    let mut line_y = 0;
    let mut path_str: String = "".to_string();

    if let Some(stem_os) = path.file_stem()
        && let Some(stem_str) = stem_os.to_str()
    {
        let weekday = chrono::NaiveDate::parse_from_str(stem_str, "%Y-%m-%d")
            .unwrap()
            .weekday();
        path_str = format!("{} {}", stem_str, weekday.to_string().as_str());
    }
    queue!(
        stdout,
        cursor::MoveTo(0, line_y),
        style::PrintStyledContent(path_str.white())
    )?;

    let init_line_y = 2;
    line_y = init_line_y;

    for (line_idx, line) in terminal_lines.iter().enumerate() {
        if line_idx >= height_index
            && line_idx < height_index + term_height as usize - init_line_y as usize
        {
            queue!(
                stdout,
                cursor::MoveTo(0, line_y),
                style::PrintStyledContent(line.clone().white())
            )?;
            line_y += 1;
        }
    }

    std::io::stdout().flush()?;

    Ok(())
}
