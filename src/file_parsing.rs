use std::path::{Path, PathBuf};

use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::io::{Error, ErrorKind};

use crate::utility;

use crate::question_structs::{Informative, Question};

use rand::seq::SliceRandom;

use xxhash_rust::xxh3::xxh3_128;

fn generate_jumbled_questions_file(
    questions_path: &Path,
    jumbled_questions_path: &Path,
) -> io::Result<()> {
    let questions_text = fs::read_to_string(questions_path)?;
    let mut questions: Vec<Question> = Vec::new();
    let mut rng = rand::rng();

    for question in questions_text
        .split("\n")
        .map(|a| <Question as From<_>>::from(a.to_string()))
    {
        if question.is_question()? {
            questions.push(question);
        }
    }
    questions.shuffle(&mut rng);

    let mut file: fs::File = OpenOptions::new()
        .append(true)
        .open(jumbled_questions_path)?;

    for quesiton in questions {
        file.write_all(quesiton.to_string().as_bytes())?;
        file.write_all("\n".as_bytes())?;
    }

    Ok(())
}

pub fn generate_jumbled_questions_file_name() -> PathBuf {
    let hash = xxh3_128(utility::get_day_file_name(0).as_bytes());
    let hashed_string = format!("{:032x}", hash)[..15].to_string();

    let file_name: PathBuf = std::path::PathBuf::from("/tmp")
        .join("jrl-".to_string() + &hashed_string + "-" + &utility::get_day_file_name(0));

    file_name
}

pub fn get_question(questions_path: &Path) -> io::Result<Question> {
    let jumbled_questions_path = generate_jumbled_questions_file_name();

    if !jumbled_questions_path.exists() {
        fs::write(&jumbled_questions_path, "")?;
    }
    if jumbled_questions_path
        .metadata()
        .map(|m| m.len())
        .unwrap_or(0)
        == 0
    {
        generate_jumbled_questions_file(questions_path, &jumbled_questions_path)?;
    }

    let all_questions = fs::read_to_string(&jumbled_questions_path)?;

    let mut file: fs::File = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&jumbled_questions_path)?;

    let number_of_quesions = all_questions.lines().count();
    let unconsumed_string: String =
        all_questions
            .lines()
            .take(number_of_quesions - 1)
            .fold(String::new(), |mut acc, line| {
                if !acc.is_empty() {
                    acc.push('\n');
                }
                acc.push_str(line);
                acc
            });

    file.write_all(unconsumed_string.as_bytes())?;

    Ok(all_questions
        .lines()
        .last()
        .ok_or(Error::new(ErrorKind::InvalidData, "no questions found"))?
        .to_string()
        .into())
}

pub fn exists_today_file(jrl_dir_path: &Path, today_file: &String) -> io::Result<bool> {
    let today_file_path = jrl_dir_path.join(today_file);

    if !jrl_dir_path.is_dir() {
        match fs::create_dir(jrl_dir_path) {
            Ok(()) => (),
            Err(e) => println!("error: {}", e),
        }
    }

    if jrl_dir_path.is_dir() {
        for entry in fs::read_dir(jrl_dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("txt")
                && path == today_file_path
            {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub fn write_question(mut file: &fs::File, question: &Question) -> io::Result<()> {
    // write info point about the question
    file.write_all(
        chrono::offset::Local::now()
            .format("i: %H:%M\n")
            .to_string()
            .as_bytes(),
    )?;

    // write the actual question
    file.write_all(question.get_type_as_str()?.as_bytes())?;
    file.write_all(": ".as_bytes())?;
    file.write_all(question.get_text()?.as_bytes())?;
    file.write_all("\n".as_bytes())?;

    Ok(())
}

pub fn write_answer(mut file: &fs::File, answer: &String) -> io::Result<()> {
    // answer might contain multiple endlines if it is copy pasted so break it into multiple answers and wrap each one individually with a: and final \n
    for line in answer.split("\n") {
        file.write_all("a: ".as_bytes())?;
        file.write_all(line.as_bytes())?;
        file.write_all("\n".as_bytes())?;
    }

    Ok(())
}
