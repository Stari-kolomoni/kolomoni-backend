use std::collections::HashMap;

use kolomoni_database::{
    entities,
    query::{ExpandedEnglishWordInfo, ExpandedSloveneWordInfo},
};
use slotmap::{new_key_type, SlotMap};
use uuid::Uuid;



new_key_type! { struct EnglishWordSlotMapKey; }
new_key_type! { struct SloveneWordSlotMapKey; }
new_key_type! { struct CategorySlotMapKey; }


/// An english word present in the search indexer cache.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CachedEnglishWord {
    /// Base english word.
    pub word: entities::word_english::Model,

    /// Categories this word belongs to.
    categories: Vec<CategorySlotMapKey>,

    /// The suggested translations belonging to this word.
    suggested_translations: Vec<SloveneWordSlotMapKey>,

    /// The translations linked to this word.
    translations: Vec<SloveneWordSlotMapKey>,
}

impl CachedEnglishWord {
    /// Obtain a [`CachedEnglishWord`] given an [`ExpandedEnglishWordInfo`]
    /// (which you can get by fetching info from the database) and access to the indexer cache.
    ///
    /// `None` can be returned if the word has invalid links to slovene words or categories
    /// (i.e. when the linked words/categories don't exist in the cache yet).
    pub fn from_expanded_database_info(
        expanded_info: ExpandedEnglishWordInfo,
        cache: &KolomoniEntityCache,
    ) -> Option<Self> {
        // We need to map categories, suggestions and translations to their slot map keys by searching by their ID.

        let mut category_keys = Vec::with_capacity(expanded_info.categories.len());
        for category in expanded_info.categories {
            let Some(category_slot_map_key) = cache.category_key(category.id) else {
                return None;
            };

            category_keys.push(category_slot_map_key);
        }

        let mut suggested_translation_keys =
            Vec::with_capacity(expanded_info.suggested_translations.len());
        for suggested_translation in expanded_info.suggested_translations {
            let Some(suggested_translation_slot_map_key) =
                cache.slovene_word_key(suggested_translation.word.word_id)
            else {
                return None;
            };

            suggested_translation_keys.push(suggested_translation_slot_map_key);
        }

        let mut translation_keys = Vec::with_capacity(expanded_info.translations.len());
        for translation in expanded_info.translations {
            let Some(translation_slot_map_key) = cache.slovene_word_key(translation.word.word_id)
            else {
                return None;
            };

            translation_keys.push(translation_slot_map_key);
        }


        Some(Self {
            word: expanded_info.word,
            categories: category_keys,
            suggested_translations: suggested_translation_keys,
            translations: translation_keys,
        })
    }

    /// Obtain the [`Uuid`] associated with this english word.
    pub fn uuid(&self) -> &Uuid {
        &self.word.word_id
    }

    /// Convert this [`CachedEnglishWord`] back into an [`ExpandedEnglishWordInfo`].
    ///
    /// This will require access to the slot maps of the indexer cache
    /// (which we need to convert weak slovene word / category references back into their explicit form).
    fn into_expanded_word_info(
        self,
        slot_context: &EntitySlotMapContext,
    ) -> Option<ExpandedEnglishWordInfo> {
        let mut categories = Vec::with_capacity(self.categories.len());
        for category_key in self.categories {
            let Some(category) = slot_context.category_slot_map.get(category_key) else {
                return None;
            };

            categories.push(category.clone().into_inner());
        }


        let mut suggested_translations = Vec::with_capacity(self.suggested_translations.len());
        for suggested_translation_key in self.suggested_translations {
            let Some(suggested_translation) = slot_context
                .slovene_word_slot_map
                .get(suggested_translation_key)
            else {
                return None;
            };

            let Some(suggested_translation) = suggested_translation
                .clone()
                .into_expanded_word_info(slot_context)
            else {
                return None;
            };

            suggested_translations.push(suggested_translation);
        }


        let mut translations = Vec::with_capacity(self.translations.len());
        for translation_key in self.translations {
            let Some(cached_translation) = slot_context.slovene_word_slot_map.get(translation_key)
            else {
                return None;
            };

            let Some(translation) = cached_translation
                .clone()
                .into_expanded_word_info(slot_context)
            else {
                return None;
            };

            translations.push(translation);
        }


        Some(ExpandedEnglishWordInfo {
            word: self.word,
            categories,
            suggested_translations,
            translations,
        })
    }
}



/// A slovene word present in the search indexer cache.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CachedSloveneWord {
    pub word: entities::word_slovene::Model,

    categories: Vec<CategorySlotMapKey>,
}

impl CachedSloveneWord {
    /// Obtain a [`CachedSloveneWord`] given an [`ExpandedSloveneWordInfo`]
    /// (which you can get by fetching info from the database) and access to the indexer cache.
    ///
    /// `None` can be returned if the word has invalid links to categories
    /// (i.e. when the categories don't exist in the cache yet).
    pub fn from_expanded_database_info(
        expanded_info: ExpandedSloveneWordInfo,
        cache: &KolomoniEntityCache,
    ) -> Option<Self> {
        // We need to map categories to their keys by searching by their ID.
        let mut category_keys = Vec::with_capacity(expanded_info.categories.len());
        for category in expanded_info.categories {
            let Some(category_slot_map_key) = cache.category_key(category.id) else {
                return None;
            };

            category_keys.push(category_slot_map_key);
        }


        Some(Self {
            word: expanded_info.word,
            categories: category_keys,
        })
    }

    /// Obtain the [`Uuid`] associated with this sloveney word.
    pub fn uuid(&self) -> &Uuid {
        &self.word.word_id
    }

    /// Convert this [`CachedSloveneWord`] back into an [`ExpandedSloveneWordInfo`].
    ///
    /// This will require access to the slot maps of the indexer cache
    /// (which we need to convert weak category references back into their explicit form).
    fn into_expanded_word_info(
        self,
        slot_context: &EntitySlotMapContext,
    ) -> Option<ExpandedSloveneWordInfo> {
        let mut categories = Vec::with_capacity(self.categories.len());
        for category_key in self.categories {
            let Some(category) = slot_context.category_slot_map.get(category_key) else {
                return None;
            };

            categories.push(category.clone().into_inner());
        }


        Some(ExpandedSloveneWordInfo {
            word: self.word,
            categories,
        })
    }
}



/// A category present in the search indexer cache.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CachedCategory {
    pub category: entities::category::Model,
}

impl CachedCategory {
    pub fn from_database_model(model: entities::category::Model) -> Self {
        Self { category: model }
    }

    pub fn id(&self) -> i32 {
        self.category.id
    }

    fn into_inner(self) -> entities::category::Model {
        self.category
    }
}


/// An internal context struct that we use to pass a set of
/// accesses to [`SlotMap`]s residing inside [`KolomoniEntityCache`].
///
/// This is an implementation detail and not visible externally.
struct EntitySlotMapContext<'e, 's, 'c> {
    #[allow(dead_code)]
    english_word_slot_map: &'e SlotMap<EnglishWordSlotMapKey, CachedEnglishWord>,

    slovene_word_slot_map: &'s SlotMap<SloveneWordSlotMapKey, CachedSloveneWord>,

    category_slot_map: &'c SlotMap<CategorySlotMapKey, CachedCategory>,
}


/// A cache containing english and slovene words as well as word categories.
///
/// All entities are stored using [`SlotMap`]s, allowing us to establish weak links between them.
///
/// For example, this allow us to simply hold weak slot map keys inside the [`translations`][CachedEnglishWord::translations]
/// `Vec` field of the english word.
/// This means that each english word weakly references corresponding slovene words
/// that are linked as translations instead of holding their entire information in themselves, improving memory usage.
/// Another upside to this approach is that modifying a slovene word is immediately reflected in all english words
/// it is a translation of, simplifying word updates.
pub struct KolomoniEntityCache {
    english_word_slot_map: SlotMap<EnglishWordSlotMapKey, CachedEnglishWord>,
    english_word_uuid_to_key_map: HashMap<Uuid, EnglishWordSlotMapKey>,

    slovene_word_slot_map: SlotMap<SloveneWordSlotMapKey, CachedSloveneWord>,
    slovene_word_uuid_to_key_map: HashMap<Uuid, SloveneWordSlotMapKey>,

    category_slot_map: SlotMap<CategorySlotMapKey, CachedCategory>,
    category_id_to_key_map: HashMap<i32, CategorySlotMapKey>,
}

impl KolomoniEntityCache {
    /// Initialize an empty entity cache.
    pub fn new() -> Self {
        let english_word_slot_map = SlotMap::<EnglishWordSlotMapKey, CachedEnglishWord>::with_key();
        let english_word_uuid_to_key_map = HashMap::new();

        let slovene_word_slot_map = SlotMap::<SloveneWordSlotMapKey, CachedSloveneWord>::with_key();
        let slovene_word_uuid_to_key_map = HashMap::new();

        let category_slot_map = SlotMap::<CategorySlotMapKey, CachedCategory>::with_key();
        let category_id_to_key_map = HashMap::new();


        Self {
            english_word_slot_map,
            english_word_uuid_to_key_map,
            slovene_word_slot_map,
            slovene_word_uuid_to_key_map,
            category_slot_map,
            category_id_to_key_map,
        }
    }

    /// Clear the entity cache.
    pub fn clear(&mut self) {
        self.english_word_slot_map.clear();
        self.english_word_uuid_to_key_map.clear();

        self.slovene_word_slot_map.clear();
        self.slovene_word_uuid_to_key_map.clear();

        self.category_slot_map.clear();
        self.category_id_to_key_map.clear();
    }


    fn slot_context(&self) -> EntitySlotMapContext {
        EntitySlotMapContext {
            english_word_slot_map: &self.english_word_slot_map,
            slovene_word_slot_map: &self.slovene_word_slot_map,
            category_slot_map: &self.category_slot_map,
        }
    }


    /// Obtain an [`ExpandedEnglishWordInfo`] given its [`Uuid`], if present in the cache and if containing valid connections.
    pub fn english_word(&self, word_uuid: Uuid) -> Option<ExpandedEnglishWordInfo> {
        let Some(english_word_slot_map_key) = self.english_word_uuid_to_key_map.get(&word_uuid)
        else {
            return None;
        };

        let Some(cached_english_word) = self.english_word_slot_map.get(*english_word_slot_map_key)
        else {
            // This can only happen if `english_word_uuid_to_key_map` and `english_word_slot_map`
            // get out of sync, which is a terrible error, and should panic.
            panic!(
                "english_word_uuid_to_key_map was mistaken about english_word_slot_map containing a key!"
            );
        };

        cached_english_word
            .clone()
            .into_expanded_word_info(&self.slot_context())
    }

    /// Obtain an [`EnglishWordSlotMapKey`] slot map key given its [`Uuid`], if present in the cache.
    #[allow(dead_code)]
    fn english_word_key(&self, word_uuid: Uuid) -> Option<EnglishWordSlotMapKey> {
        self.english_word_uuid_to_key_map.get(&word_uuid).copied()
    }

    /// Insert (or overwrite) an english word into the entity cache.
    ///
    /// For conversion from [`ExpandedEnglishWordInfo`] to [`CachedEnglishWord`],
    /// see [`CachedEnglishWord::from_expanded_database_info`].
    pub fn insert_or_update_english_word(&mut self, english_word: CachedEnglishWord) {
        let word_uuid = *english_word.uuid();

        if let Some(existing_slot_map_key) = self.english_word_uuid_to_key_map.get(&word_uuid) {
            // Update the existing word.

            let Some(existing_word_entry) =
                self.english_word_slot_map.get_mut(*existing_slot_map_key)
            else {
                // If the logic is valid, this should never occur.
                // The only way this can happen is if `english_word_uuid_to_key_map` is
                // not kept up to date. Either way, this is an incredibly bad oversight and should panic.
                panic!(
                    "existing_word_entry does not exist in english_word_slot_map, \
                    even though english_word_uuid_to_key_map contains its key?!"
                );
            };

            *existing_word_entry = english_word;
            return;
        }


        // Insert a fresh word (don't forget to update `english_word_uuid_to_key_map`)!

        let new_key = self.english_word_slot_map.insert(english_word);
        self.english_word_uuid_to_key_map.insert(word_uuid, new_key);
    }

    /// Remove an english word from the entity cache.
    ///
    /// Returns `Err(())` if the word wasn't present if the cache.
    pub fn remove_english_word(&mut self, word_uuid: Uuid) -> Result<(), ()> {
        let Some(english_word_slot_map_key) = self.english_word_uuid_to_key_map.get(&word_uuid)
        else {
            return Err(());
        };

        let Some(_) = self
            .english_word_slot_map
            .remove(*english_word_slot_map_key)
        else {
            panic!(
                "english_word_slot_map_key was in reality not present in english_word_slot_map, \
                even though english_word_uuid_to_key_map indicated that it is!"
            )
        };

        self.english_word_uuid_to_key_map.remove(&word_uuid);

        Ok(())
    }


    /// Obtain an [`ExpandedSloveneWordInfo`] given its [`Uuid`], if present in the cache and if containing valid connections.
    pub fn slovene_word(&self, word_uuid: Uuid) -> Option<ExpandedSloveneWordInfo> {
        let Some(slovene_word_slot_map_key) = self.slovene_word_uuid_to_key_map.get(&word_uuid)
        else {
            return None;
        };

        let Some(cached_slovene_word) = self.slovene_word_slot_map.get(*slovene_word_slot_map_key)
        else {
            // This can only happen if `slovene_word_uuid_to_key_map` and `slovene_word_slot_map`
            // get out of sync, which is a terrible error, and should panic.
            panic!(
                "slovene_word_uuid_to_key_map was mistaken about slovene_word_slot_map containing a key!"
            );
        };

        cached_slovene_word
            .clone()
            .into_expanded_word_info(&self.slot_context())
    }

    /// Obtain an [`SloveneWordSlotMapKey`] slot map key given its [`Uuid`], if present in the cache.
    fn slovene_word_key(&self, word_uuid: Uuid) -> Option<SloveneWordSlotMapKey> {
        self.slovene_word_uuid_to_key_map.get(&word_uuid).copied()
    }

    /// Insert (or overwrite) a slovene word into the entity cache.
    ///
    /// For conversion from [`ExpandedSloveneWordInfo`] to [`CachedSloveneWord`],
    /// see [`CachedSloveneWord::from_expanded_database_info`].
    pub fn insert_or_update_slovene_word(&mut self, slovene_word: CachedSloveneWord) {
        let word_uuid = *slovene_word.uuid();

        if let Some(existing_slot_map_key) = self.slovene_word_uuid_to_key_map.get(&word_uuid) {
            // Update the existing word.

            let Some(existing_word_entry) =
                self.slovene_word_slot_map.get_mut(*existing_slot_map_key)
            else {
                // If the logic is valid, this should never occur.
                // The only way this can happen is if `slovene_word_uuid_to_key_map` is
                // not kept up to date. Either way, this is an incredibly bad oversight and should panic.
                panic!(
                    "existing_word_entry does not exist in slovene_word_slot_map, \
                    even though slovene_word_uuid_to_key_map contains its key?!"
                );
            };

            *existing_word_entry = slovene_word;
            return;
        }


        // Insert a fresh word (don't forget to update `slovene_word_uuid_to_key_map`)!

        let new_key = self.slovene_word_slot_map.insert(slovene_word);
        self.slovene_word_uuid_to_key_map.insert(word_uuid, new_key);
    }

    /// Remove a slovene word from the entity cache.
    ///
    /// Returns `Err(())` if the word wasn't present if the cache.
    pub fn remove_slovene_word(&mut self, word_uuid: Uuid) -> Result<(), ()> {
        let Some(slovene_word_slot_map_key) = self.slovene_word_uuid_to_key_map.get(&word_uuid)
        else {
            return Err(());
        };

        let Some(_) = self
            .slovene_word_slot_map
            .remove(*slovene_word_slot_map_key)
        else {
            panic!(
                "slovene_word_slot_map_key was in reality not present in slovene_word_slot_map, \
                even though slovene_word_uuid_to_key_map indicated that it is!"
            )
        };

        self.slovene_word_uuid_to_key_map.remove(&word_uuid);

        Ok(())
    }


    /// Obtain information about a category given its ID, if present in the cache.
    #[allow(dead_code)]
    pub fn category(&self, category_id: i32) -> Option<entities::category::Model> {
        let Some(category_slot_map_key) = self.category_id_to_key_map.get(&category_id) else {
            return None;
        };

        let Some(cached_category) = self.category_slot_map.get(*category_slot_map_key) else {
            // This can only happen if `category_id_to_key_map` and `category_slot_map`
            // get out of sync, which is a terrible error, and should panic.
            panic!("category_id_to_key_map was mistaken about category_slot_map containing a key!");
        };

        Some(cached_category.clone().into_inner())
    }

    /// Obtain a [`CategorySlotMapKey`] slot map key given the category's ID, if present in the cache.
    fn category_key(&self, category_id: i32) -> Option<CategorySlotMapKey> {
        self.category_id_to_key_map.get(&category_id).copied()
    }

    /// Insert (or overwrite) a category into the entity cache.
    ///
    /// For conversion from the database category model to [`CachedCategory`],
    /// see [`CachedCategory::from_database_model`].
    pub fn insert_or_update_category(&mut self, category: CachedCategory) {
        let category_id = category.id();

        if let Some(existing_slot_map_key) = self.category_id_to_key_map.get(&category_id) {
            // Update the existing word.

            let Some(existing_category_entry) =
                self.category_slot_map.get_mut(*existing_slot_map_key)
            else {
                // If the logic is valid, this should never occur.
                // The only way this can happen is if `category_id_to_key_map` is
                // not kept up to date. Either way, this is an incredibly bad oversight and should panic.
                panic!(
                    "existing_category_entry does not exist in category_slot_map, \
                    even though category_id_to_key_map contains its key?!"
                );
            };

            *existing_category_entry = category;
            return;
        }


        // Insert a fresh word (don't forget to update `category_id_to_key_map`)!

        let new_key = self.category_slot_map.insert(category);
        self.category_id_to_key_map.insert(category_id, new_key);
    }

    /// Remove a category from the entity cache.
    ///
    /// Returns `Err(())` if the category wasn't present if the cache.
    pub fn remove_category(&mut self, category_id: i32) -> Result<(), ()> {
        let Some(category_slot_map_key) = self.category_id_to_key_map.get(&category_id) else {
            return Err(());
        };

        let Some(_) = self.category_slot_map.remove(*category_slot_map_key) else {
            panic!(
                "category_slot_map was in reality not present in category_slot_map, \
                even though category_id_to_key_map indicated that it is!"
            )
        };

        self.category_id_to_key_map.remove(&category_id);

        Ok(())
    }
}
