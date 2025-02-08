use chrono::Utc;
use kolomoni_api_client::{api::dictionary::{english::{EnglishWordMeaningToCreate, EnglishWordToCreate}, slovene::{SloveneWordMeaningToCreate, SloveneWordToCreate}}, AuthenticatedClient, SharedApiClientEndpointGroups};
use kolomoni_core::api_models::{EnglishWordWithMeanings, SloveneWordWithMeanings};


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

    pub fn abbreviation(&self) -> Option<&'static str> {
        match self {
            SampleEnglishWord::Ability => None,
            SampleEnglishWord::Charisma => Some("CHA"),
            SampleEnglishWord::Attack => None,
            SampleEnglishWord::CriticalHit => Some("crit"),
            SampleEnglishWord::HitPoints => Some("HP"),
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
    
    pub async fn create(
        &self,
        client: &AuthenticatedClient,
    ) -> EnglishWordWithMeanings {
        let before_word_creation = Utc::now();

        let new_word = client.english_dictionary().create_english_word(EnglishWordToCreate {
            lemma: self.lemma().to_owned()
        }).await.expect("failed to create sample english word");

        assert_eq!(self.lemma(), new_word.lemma);
        assert!(new_word.meanings.is_empty());
        assert!(new_word.created_at >= before_word_creation);
        assert_eq!(new_word.created_at, new_word.last_modified_at);


        let before_word_meaning_creation = Utc::now();

        let new_word_meaning = client.english_dictionary().create_english_word_meaning(
            new_word.id,
            EnglishWordMeaningToCreate {
                abbreviation: self.abbreviation().map(ToOwned::to_owned),
                disambiguation: self.disambiguation().map(ToOwned::to_owned),
                description: self.description().map(ToOwned::to_owned)
            }
        ).await.expect("failed to create sample english word meaning");

        assert!(new_word_meaning.created_at >= before_word_meaning_creation);
        assert_eq!(new_word_meaning.created_at, new_word_meaning.last_modified_at);
        assert_eq!(self.abbreviation(), new_word_meaning.abbreviation.as_deref());
        assert_eq!(self.disambiguation(), new_word_meaning.disambiguation.as_deref());
        assert_eq!(self.description(), new_word_meaning.description.as_deref());
        
        
        let new_word_including_meanings = client.english_dictionary().english_word_by_id(
            new_word.id
        ).await.expect("failed to re-fetch full sample english word with meanings");


        assert!(new_word_including_meanings.meanings.len() == 1);

        let first_meaning = &new_word_including_meanings.meanings[0];

        assert_eq!(first_meaning.word_meaning_id, new_word_meaning.word_meaning_id);
        assert_eq!(first_meaning.disambiguation, new_word_meaning.disambiguation);
        assert_eq!(first_meaning.abbreviation, new_word_meaning.abbreviation);
        assert_eq!(first_meaning.description, new_word_meaning.description);
        assert_eq!(first_meaning.created_at, new_word_meaning.created_at);
        assert_eq!(first_meaning.last_modified_at, new_word_meaning.last_modified_at);

        assert!(first_meaning.categories.is_empty());
        assert!(first_meaning.translations.is_empty());


        new_word_including_meanings
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

    pub fn abbreviation(&self) -> Option<&'static str> {
        match self {
            SampleSloveneWord::Sposobnost => None,
            SampleSloveneWord::Karizma => None,
            SampleSloveneWord::Napad => None,
            SampleSloveneWord::Terna => None,
            SampleSloveneWord::KriticniIzid => None,
            SampleSloveneWord::UsodniZadetek => None,
            SampleSloveneWord::ZivljenskaTocka => Some("ZT"),
            SampleSloveneWord::Zdravje => None,
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

    pub async fn create(
        &self,
        client: &AuthenticatedClient,
    ) -> SloveneWordWithMeanings {
        let before_word_creation = Utc::now();

        let new_word = client.slovene_dictionary().create_slovene_word(
            SloveneWordToCreate {
                lemma: self.lemma().to_owned()
            }
        ).await.expect("failed to create sample slovene word");

        assert_eq!(self.lemma(), new_word.lemma);
        assert!(new_word.meanings.is_empty());
        assert!(new_word.created_at >= before_word_creation);
        assert_eq!(new_word.created_at, new_word.last_modified_at);


        let before_word_meaning_creation = Utc::now();

        let new_word_meaning = client.slovene_dictionary().create_slovene_word_meaning(
            new_word.id,
            SloveneWordMeaningToCreate {
                abbreviation: self.abbreviation().map(ToOwned::to_owned),
                description: self.description().map(ToOwned::to_owned),
                disambiguation: self.disambiguation().map(ToOwned::to_owned)
            }
        ).await.expect("failed to create sample slovene word meaning");

        assert!(new_word_meaning.created_at >= before_word_meaning_creation);
        assert_eq!(new_word_meaning.created_at, new_word_meaning.last_modified_at);
        assert_eq!(self.abbreviation(), new_word_meaning.abbreviation.as_deref());
        assert_eq!(self.disambiguation(), new_word_meaning.disambiguation.as_deref());
        assert_eq!(self.description(), new_word_meaning.description.as_deref());


        let new_word_including_meanings = client.slovene_dictionary().slovene_word_by_id(
            new_word.id
        ).await.expect("failed to re-fetch full sample slovene word with meanings");


        assert!(new_word_including_meanings.meanings.len() == 1);

        let first_meaning = &new_word_including_meanings.meanings[0];

        assert_eq!(first_meaning.word_meaning_id, new_word_meaning.word_meaning_id);
        assert_eq!(first_meaning.disambiguation, new_word_meaning.disambiguation);
        assert_eq!(first_meaning.abbreviation, new_word_meaning.abbreviation);
        assert_eq!(first_meaning.description, new_word_meaning.description);
        assert_eq!(first_meaning.created_at, new_word_meaning.created_at);
        assert_eq!(first_meaning.last_modified_at, new_word_meaning.last_modified_at);

        assert!(first_meaning.categories.is_empty());
        assert!(first_meaning.translations.is_empty());


        new_word_including_meanings
    }
}
