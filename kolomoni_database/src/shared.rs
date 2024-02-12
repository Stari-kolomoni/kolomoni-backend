use thiserror::Error;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Error, Debug)]
pub enum WordLanguageError {
    #[error("unrecognized language: {language}")]
    UnrecognizedLanguage { language: String },
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WordLanguage {
    English,
    Slovene,
}

impl WordLanguage {
    pub fn from_ietf_language_tag(language_tag: &str) -> Result<Self, WordLanguageError> {
        match language_tag {
            "en" => Ok(Self::English),
            "si" => Ok(Self::Slovene),
            _ => Err(WordLanguageError::UnrecognizedLanguage {
                language: language_tag.to_string(),
            }),
        }
    }

    pub fn to_ietf_language_tag(self) -> &'static str {
        match self {
            WordLanguage::English => "en",
            WordLanguage::Slovene => "si",
        }
    }
}

#[inline]
pub fn generate_random_word_uuid() -> Uuid {
    Uuid::new_v7(Timestamp::now(NoContext))
}
