use std::{collections::BTreeMap, io::Write};

use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal,
};

use crate::question_structs::{
    ChunkParser, Informative, PromptQuestionType, QuestionChunk, QuestionType,
};

// const format_error_chunk: QuestionChunk  = QuestionChunk::new();

pub fn format_content(content: &String) -> std::io::Result<String> {
    let mut invalid_chunks_number = 0;
    let mut notes: Vec<String> = Vec::new();
    let mut descriptions: Vec<String> = Vec::new();
    let mut ratings: Vec<String> = Vec::new();
    let mut long_questions: BTreeMap<String, Vec<String>>= BTreeMap::new();
    let mut short_questions: BTreeMap<String, Vec<String>>= BTreeMap::new();
    let mut this_chunk_str: String = "".to_string();

    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if !line.is_empty() {
            this_chunk_str += line;
            this_chunk_str += "\n";
        }
        if (line.is_empty() || lines.peek().is_none()) && !this_chunk_str.trim().is_empty() {

            this_chunk_str = this_chunk_str.trim().to_string();
            let this_chunk = QuestionChunk::from(this_chunk_str.clone());

            let chunk_type = this_chunk.get_type().unwrap_or_else(|_| {
                invalid_chunks_number += 1;

                QuestionType::Empty
            });

            // invalid might as well be empty
            if chunk_type != QuestionType::Empty {
                let mut answer_iter = this_chunk.get_answer()?.into_iter();
                let info = this_chunk.get_informative()?.get_text()?;
                let prompt_type = this_chunk.get_prompt_type()?;

                if prompt_type == PromptQuestionType::Rating {
                    while let Some(question) = answer_iter.next() {
                        ratings.push(format!("[{}] {}", info, question.get_text()?));
                    }
                } 
                else if prompt_type == PromptQuestionType::Description {
                    if let Some(question) = answer_iter.next() {
                        descriptions.push(format!("    [{}] {}", info, question.get_text()?));
                    }

                    while let Some(question) = answer_iter.next() {
                        descriptions.push(format!("{}", question.get_text()?));
                    }
                } 
                else if prompt_type == PromptQuestionType::Note {
                    if let Some(question) = answer_iter.next() {
                        notes.push(format!("    [{}] {}", info, question.get_text()?));
                    }
                    while let Some(question) = answer_iter.next() {
                        notes.push(format!("{}", question.get_text()?));
                    }
                } 
                else if chunk_type == QuestionType::Long {
                    let question_text = this_chunk.get_question()?.get_text()?;
                    let mut answer: String = "".to_string();

                    if let Some(question) = answer_iter.next() {
                        let answer_line = format!("    [{}] {}", info, question.get_text()?); 
                        answer += answer_line.as_str();
                    }

                    while let Some(question) = answer_iter.next() {
                        let answer_line = format!("\n{}", question.get_text()?); 
                        answer += answer_line.as_str();
                    }

                    long_questions.entry(question_text)
                        .or_insert(Vec::new())
                        .push(answer);
 
                }
                else {
                    while let Some(question) = answer_iter.next() {
                        let question_text = this_chunk.get_question()?.get_text()?;
                        short_questions.entry(question_text)
                            .or_insert(Vec::new())
                            .push(format!("[{}] {}", info, question.get_text()?));
                    }
                }
            }

            this_chunk_str = "".to_string();
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
        return_content +=
            format!("Encountered {} invalid chunk(s) in file!", invalid_chunks_number).as_str();
    }

    Ok(return_content)
}

pub fn parse_display_text(content: &String) -> std::io::Result<Vec<String>> {
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
    path: &std::path::PathBuf,
    height_index: usize,
    terminal_lines: &Vec<String>,
    mut stdout: &std::io::Stdout,
) -> std::io::Result<()> {
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    let (_, term_height) = terminal::size()?;
    let mut line_y = 0;
    let mut path_str = "";

    if let Some(stem_os) = path.file_stem() {
        if let Some(stem_str) = stem_os.to_str() {
            path_str = stem_str;
        }
    }
    queue!(
        stdout,
        cursor::MoveTo(0 as u16, line_y),
        style::PrintStyledContent(path_str.white())
    )?;

    let init_line_y = 2;
    line_y = init_line_y;

    let mut line_idx = 0;
    for line in terminal_lines {
        if line_idx >= height_index
            && line_idx < height_index + term_height as usize - init_line_y as usize
        {
            queue!(
                stdout,
                cursor::MoveTo(0 as u16, line_y),
                style::PrintStyledContent(line.clone().white())
            )?;
            line_y += 1;
        }
        line_idx += 1;
    }

    std::io::stdout().flush()?;

    Ok(())
}
