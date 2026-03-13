use crate::{loops::get_jrl_files, question_structs::{ChunkParser, Informative, PromptQuestionType}};
use std::{collections::BTreeMap, path::PathBuf};
use std::io;
use std::fs;
use crate::pager;

pub fn get_statistics(jrl_dir_path: PathBuf, entries_to_consider: usize) -> io::Result<()> {
    let mut word_dict: BTreeMap<String, i64>= BTreeMap::new();
    let all_files = get_jrl_files(&jrl_dir_path)?;

    let recent_entries = &all_files[all_files.len().saturating_sub(entries_to_consider)..];

    let mut best_rating_opt: Option<f64> = None;
    let mut best_rating_day: String = "".to_string();
    let mut best_description: String = "".to_string();
    
    let mut worst_rating_opt: Option<f64> = None;
    let mut worst_rating_day: String = "".to_string();
    let mut worst_description: String = "".to_string();

    let mut longest_text_rating_opt: Option<f64> = None;
    let mut longest_text_day: String = "".to_string();
    let mut longest_description: String = "".to_string();

    let mut average_rating: f64 = 0.0;
    let mut files_rated: f64 = 0.0;

    let mut entries_number = 0;

    for file in recent_entries {
        let  file_content = fs::read_to_string(file)?;
        entries_number += 1;
        let mut lines = file_content.lines().peekable();

        let mut description: String = "".to_string();
        let mut final_file_rating_opt: Option<f64> = None;

        loop {
            match pager::get_next_chunk(&mut lines) {
                Ok(this_chunk) => {
                    // collect statistics on answer words (how often they are used)
                    let answer_vec = this_chunk.get_answer()?;
                    let mut answer_words: Vec<String> = Vec::new(); 
                    for question in answer_vec {
                        let line_words = question
                            .question
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());

                        answer_words.extend(line_words);
                    }
                    for word in answer_words {
                        let lower_word = word.to_lowercase();
                        word_dict.entry(lower_word.to_string())
                            .and_modify(|i| *i += 1)
                            .or_insert(1);
                    }

                    // see which days were the best/worst and store the rating and description
                    // let chunk_type = this_chunk.get_type().unwrap_or_else(|_| { QuestionType::Empty});
                    let prompt_type = this_chunk.get_prompt_type()?;
                    let mut answer_iter = this_chunk.get_answer()?.into_iter();
                    let info = this_chunk.get_informative()?.get_text()?;
                    if prompt_type == PromptQuestionType::Rating {

                        match this_chunk.get_answer()?[0].get_text()?.as_str().parse::<f64>() {
                            Ok(rating) => {
                                final_file_rating_opt = Some(rating);
                            }
                            Err(_) => {}
                        }
                    }
                    else if prompt_type == PromptQuestionType::Description {
                        if let Some(question) = answer_iter.next() {
                            description += format!("\n    [{}] {}", info, question.get_text()?).as_str();
                        }

                        while let Some(question) = answer_iter.next() {
                            description += format!("\n    {}", question.get_text()?).as_str();
                        }
                    } 
                }
                Err(pager::ChunkError::UnexpectedFileEnd) => {
                    break;
                }
                Err(_) => { }
            }
        }

        description = pager::parse_display_text(&description)?.join("\n");
        if let Some(file_rating) = final_file_rating_opt{
            files_rated += 1.0;
            average_rating += file_rating;
            
            if best_rating_opt.map_or(true, |r| r < file_rating) {
                best_rating_opt= Some(file_rating);
                if let Some(file_name) = file.file_stem(){
                    best_rating_day = file_name.to_string_lossy().to_string();
                    best_rating_day = best_rating_day.replace("-", ".");
                }
                best_description = description.clone();
            }
            if worst_rating_opt.map_or(true,  | r| r > file_rating) {
                worst_rating_opt = Some(file_rating);
                if let Some(file_name) = file.file_stem() {
                    worst_rating_day = file_name.to_string_lossy().to_string();
                    worst_rating_day = worst_rating_day.replace("-", ".");
                }
                worst_description = description.clone();
            }
        }
        if description.len() > longest_description.len() {
            longest_text_rating_opt = final_file_rating_opt;
            if let Some(file_name) = file.file_stem() {
                longest_text_day = file_name.to_string_lossy().to_string();
                longest_text_day = longest_text_day.replace("-", ".");
            }
            longest_description = description.clone();
        }
        
    }
    println!("Analytics for the past {} entries:\n", entries_number);

    if best_rating_opt == None { 
        println!("No ratings given for the past {} days.", entries_number);
    }
    else if best_rating_opt == worst_rating_opt {
        if let Some(best_rating) = best_rating_opt{
            println!("{} was your best and worst day with {} rating:", best_rating_day, best_rating);
            println!("{}", best_description);
        }
    }
    else {
        let average_rating = average_rating / files_rated;
        println!("Average day rating: {:.2}\n", average_rating);

        if let Some(best_rating) = best_rating_opt{
            println!("{} was your best day with {} rating:", best_rating_day, best_rating);
            println!("{}", best_description);
        }

        if let Some(worst_rating) = worst_rating_opt{
            println!("{} was your worst day with {} rating:", worst_rating_day, worst_rating);
            println!("{}", worst_description);
        }
    }

    let max_word_len = 20;
    let mut most_common_words: Vec<(&str, i64)> = vec![("", 0); max_word_len + 1];
    
    for (word, count) in word_dict.iter() {
        let word_len = word.len();
        if word_len > 0 && word_len <= max_word_len{
            if most_common_words[word_len].1 < *count {
                most_common_words[word_len] = (word, *count);
            }
        }
    }

    for i in 0..max_word_len {
        let (word, len) = most_common_words[i];
        if len != 0 {
            println!("Most common {} letter word was {}, used {} times", i, word, len); 
        }
    }
    if let Some(longest_text_rating) = longest_text_rating_opt {
        println!("\n{} was your longest input day with {} rating:", longest_text_day, longest_text_rating);
        println!("{}", longest_description);
    }
    else{
        println!("\n{} was your longest input day:", longest_text_day);
        println!("{}", longest_description);
    }

    Ok(())
}