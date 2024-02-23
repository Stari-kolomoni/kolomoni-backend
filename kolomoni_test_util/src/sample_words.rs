use http::{Method, StatusCode};
use kolomoni::api::v1::dictionary::{english_word::{EnglishWord, EnglishWordCreationRequest, EnglishWordCreationResponse}, slovene_word::{SloveneWord, SloveneWordCreationRequest, SloveneWordCreationResponse}, suggestions::TranslationSuggestionRequest, translations::TranslationRequest};

use crate::TestServer;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SampleEnglishWord {
    Ability,
    Charisma,
    Attack,
    CriticalHit,
    HitPoints,
}

impl SampleEnglishWord {
    pub fn lemma(&self) -> &'static str {
        match self {
            SampleEnglishWord::Ability => "ability",
            SampleEnglishWord::Charisma => "charisma",
            SampleEnglishWord::Attack => "attack",
            SampleEnglishWord::CriticalHit => "critical hit",
            SampleEnglishWord::HitPoints => "hit points",
        }
    }

    pub fn disambiguation(&self) -> Option<&'static str> {
        match self {
            SampleEnglishWord::Ability => None,
            SampleEnglishWord::Charisma => Some("mechanical ability"),
            SampleEnglishWord::Attack => Some("in combat"),
            SampleEnglishWord::CriticalHit => None,
            SampleEnglishWord::HitPoints => Some("game mechanic"),
        }
    }

    #[rustfmt::skip]
    pub fn description(&self) -> Option<&'static str> {
        match self {
            SampleEnglishWord::Ability => 
                Some("A creature's assets as well as weaknesses."),
            SampleEnglishWord::Charisma => 
                Some("A measuring force of Personality."),
            SampleEnglishWord::Attack => 
                None,
            SampleEnglishWord::CriticalHit => 
                Some("When a player rolls a natural 20 on a check, save, or attack roll."),
            SampleEnglishWord::HitPoints => 
                Some(
                    "A character's hit points define how tough your character is \
                    in combat and other dangerous situations."
                ),
        }
    }
}



pub enum SampleSloveneWord {
    Sposobnost,
    Karizma,
    Napad,
    Terna,
    KriticniIzid,
    UsodniZadetek,
    ZivljenskaTocka,
    Zdravje,
}

impl SampleSloveneWord {
    pub fn lemma(&self) -> &'static str {
        match self {
            SampleSloveneWord::Sposobnost => "sposobnost",
            SampleSloveneWord::Karizma => "karizma",
            SampleSloveneWord::Napad => "napad",
            SampleSloveneWord::Terna => "terna",
            SampleSloveneWord::KriticniIzid => "kritični izid",
            SampleSloveneWord::UsodniZadetek => "usodni zadetek",
            SampleSloveneWord::ZivljenskaTocka => "življenska točka",
            SampleSloveneWord::Zdravje => "zdravje",
        }
    }

    pub fn disambiguation(&self) -> Option<&'static str> {
        match self {
            SampleSloveneWord::Sposobnost => None,
            SampleSloveneWord::Karizma => None,
            SampleSloveneWord::Napad => None,
            SampleSloveneWord::Terna => None,
            SampleSloveneWord::KriticniIzid => Some("met kocke"),
            SampleSloveneWord::UsodniZadetek => None,
            SampleSloveneWord::ZivljenskaTocka => None,
            SampleSloveneWord::Zdravje => Some("igralna mehanika"),
        }
    }

    #[rustfmt::skip]
    pub fn description(&self) -> Option<&'static str> {
        match self {
            SampleSloveneWord::Sposobnost => None,
            SampleSloveneWord::Karizma => 
                Some("Moč osebnosti, sposobnost za vodenje drugih."),
            SampleSloveneWord::Napad => None,
            SampleSloveneWord::Terna => None,
            SampleSloveneWord::KriticniIzid => None,
            SampleSloveneWord::UsodniZadetek => None,
            SampleSloveneWord::ZivljenskaTocka => None,
            SampleSloveneWord::Zdravje => None,
        }
    }
}


pub async fn create_sample_english_word(
    server: &TestServer,
    access_token: &str,
    sample_word: SampleEnglishWord,
) -> EnglishWord {
    let creation_response = server.request(
        Method::POST,
        "/api/v1/dictionary/english",
    )
        .with_json_body(EnglishWordCreationRequest {
            lemma: sample_word.lemma().to_string(),
            disambiguation: sample_word.disambiguation().map(str::to_string),
            description: sample_word.description().map(str::to_string)
        })
        .with_access_token(access_token)
        .send()
        .await;

    creation_response.assert_status_equals(StatusCode::OK);

    let response_body = creation_response.json_body::<EnglishWordCreationResponse>();

    response_body.word
}

pub async fn create_sample_slovene_word(
    server: &TestServer,
    access_token: &str,
    sample_word: SampleSloveneWord,
) -> SloveneWord {
    let creation_response = server.request(
        Method::POST,
        "/api/v1/dictionary/slovene",
    )
        .with_json_body(SloveneWordCreationRequest {
            lemma: sample_word.lemma().to_string(),
            disambiguation: sample_word.disambiguation().map(str::to_string),
            description: sample_word.description().map(str::to_string)
        })
        .with_access_token(access_token)
        .send()
        .await;

    creation_response.assert_status_equals(StatusCode::OK);

    let response_body = creation_response.json_body::<SloveneWordCreationResponse>();

    response_body.word
}

pub async fn link_word_as_translation(
    server: &TestServer,
    access_token: &str,
    english_word_id: &str,
    slovene_word_id: &str,
) {
    let translation_response = server.request(
        Method::POST,
        "/api/v1/dictionary/translation"
    )
        .with_json_body(TranslationRequest {
            english_word_id: english_word_id.to_string(),
            slovene_word_id: slovene_word_id.to_string(),
        })
        .with_access_token(access_token)
        .send()
        .await;

    translation_response.assert_status_equals(StatusCode::OK);
}

pub async fn link_word_as_suggested_translation(
    server: &TestServer,
    access_token: &str,
    english_word_id: &str,
    slovene_word_id: &str,
) {
    let suggestion_response = server.request(
        Method::POST,
        "/api/v1/dictionary/suggestion"
    )
        .with_json_body(TranslationSuggestionRequest {
            english_word_id: english_word_id.to_string(),
            slovene_word_id: slovene_word_id.to_string(),
        })
        .with_access_token(access_token)
        .send()
        .await;

    suggestion_response.assert_status_equals(StatusCode::OK);
}

