// implement informative for question chunk
// do pager

use crate::cli;
use std::fmt;

const QUESTION_TYPE_TRAIL: &str = ": ";
const QUESTION_TYPE_ERROR: &str = "question chunk does not fit the format";

#[derive(PartialEq)]
pub enum QuestionType {
    Short,
    Long,
    Answer,
    Info,
    Empty,
}

#[derive(PartialEq)]
pub enum PromptQuestionType {
    Regular,
    Description,
    Note,
    Rating,
}

#[derive(PartialEq, Default, Clone, Debug)]
pub struct Question {
    pub question: String,
}

pub struct QuestionChunk {
    pub question_chunk: String,
}

impl From<String> for Question {
    fn from(s: String) -> Self {
        Question { question: s }
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}

impl From<&str> for Question {
    fn from(s: &str) -> Self {
        Question {
            question: s.to_string(),
        }
    }
}

impl From<String> for QuestionChunk {
    fn from(s: String) -> Self {
        QuestionChunk { question_chunk: s }
    }
}

impl From<&str> for QuestionChunk {
    fn from(s: &str) -> Self {
        QuestionChunk {
            question_chunk: s.to_string(),
        }
    }
}

pub trait Informative {
    fn is_question(&self) -> std::io::Result<bool>;
    fn get_type(&self) -> std::io::Result<QuestionType>;
    fn get_type_as_str(&self) -> std::io::Result<String>;
    fn get_prompt_type(&self) -> std::io::Result<PromptQuestionType>;
    fn get_text(&self) -> std::io::Result<String>;
}

pub trait ChunkParser {
    fn get_informative(&self) -> std::io::Result<Question>;
    fn get_question(&self) -> std::io::Result<Question>;
    fn get_answer(&self) -> std::io::Result<Vec<Question>>;
}

impl Informative for QuestionChunk {
    fn is_question(&self) -> std::io::Result<bool> {
        self.get_question()?.is_question()
    }

    fn get_type(&self) -> std::io::Result<QuestionType> {
        self.get_question()?.get_type()
    }

    fn get_type_as_str(&self) -> std::io::Result<String> {
        self.get_question()?.get_type_as_str()
    }

    fn get_prompt_type(&self) -> std::io::Result<PromptQuestionType> {
        self.get_question()?.get_prompt_type()
    }

    fn get_text(&self) -> std::io::Result<String> {
        Ok(self.question_chunk.clone())
    }
}

impl Informative for Question {
    fn is_question(&self) -> std::io::Result<bool> {
        let question_type = self.get_type()?;
        if question_type != QuestionType::Empty {
            return Ok(true);
        }

        Ok(false)
    }

    fn get_type(&self) -> std::io::Result<QuestionType> {
        let question_type = self.question.get(..1).unwrap_or("");
        let follow_up_chars = self.question.get(1..3).unwrap_or("");

        if follow_up_chars != QUESTION_TYPE_TRAIL {
            return Ok(QuestionType::Empty);
        }

        match question_type {
            "l" => Ok(QuestionType::Long),
            "s" => Ok(QuestionType::Short),
            "a" => Ok(QuestionType::Answer),
            "i" => Ok(QuestionType::Info),
            _ => Ok(QuestionType::Empty),
        }
    }

    fn get_type_as_str(&self) -> std::io::Result<String> {
        let question_type = self.question.get(..1).unwrap_or("");
        let follow_up_chars = self.question.get(1..3).unwrap_or("");

        if follow_up_chars != QUESTION_TYPE_TRAIL {
            return Ok("".to_string());
        }

        Ok(question_type.to_string())
    }

    fn get_prompt_type(&self) -> std::io::Result<PromptQuestionType> {
        let question_string = &self.question;

        if question_string == cli::DESCRIPTION_QUESTION_STR {
            return Ok(PromptQuestionType::Description);
        }
        if question_string == cli::NOTE_QUESTION_STR {
            return Ok(PromptQuestionType::Note);
        }
        if question_string == cli::RATING_QUESTION_STR {
            return Ok(PromptQuestionType::Rating);
        }

        Ok(PromptQuestionType::Regular)
    }

    fn get_text(&self) -> std::io::Result<String> {
        if self.get_type()? == QuestionType::Empty {
            return Ok("".to_string());
        }

        let text = self.question.get(3..).unwrap_or("");
        Ok(text.to_string())
    }
}

impl ChunkParser for QuestionChunk {
    fn get_informative(&self) -> std::io::Result<Question> {
        let question_lines: Vec<&str> = self.question_chunk.lines().collect();
        if question_lines.len() >= 3 {
            return Ok(question_lines[0].to_string().into());
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            QUESTION_TYPE_ERROR,
        ))
    }

    fn get_question(&self) -> std::io::Result<Question> {
        let question_lines: Vec<&str> = self.question_chunk.lines().collect();
        if question_lines.len() >= 3 {
            return Ok(question_lines[1].to_string().into());
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            QUESTION_TYPE_ERROR,
        ))
    }

    fn get_answer(&self) -> std::io::Result<Vec<Question>> {
        let all_lines: Vec<&str> = self.question_chunk.lines().skip(2).collect();

        if all_lines.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                QUESTION_TYPE_ERROR,
            ));
        }

        Ok(all_lines.into_iter().map(Question::from).collect())
    }
}
