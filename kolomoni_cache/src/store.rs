use std::{
    collections::{hash_map, HashMap},
    fmt::Debug,
    hash::Hash,
};


/// Describes an occupied [`EntityStore`] entry.
pub struct OccupiedStoreEntry<'s, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    store: &'s mut EntityStore<I, E>,

    target_entity_id: I,
}

impl<I, E> OccupiedStoreEntry<'_, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    /// Returns the primary ID that belongs to this entity.
    pub fn id(&self) -> &I {
        &self.target_entity_id
    }

    /// Returns a reference to the entity.
    pub fn get(&self) -> &E {
        self.store.id_to_entity_map.get(&self.target_entity_id)
            // PANIC SAFETY: This can never panic, because this struct mutably borrows from
            // the parent [`EntityStore`], ensuring no other modifications can be made to it 
            // as long as this struct lives.
            .expect("expected the entity primary ID to be still valid")
    }

    /// Returns a mutable reference to the entity.
    pub fn get_mut(&mut self) -> &mut E {
        self.store.id_to_entity_map.get_mut(&self.target_entity_id)
            // PANIC SAFETY: This can never panic, because this struct mutably borrows from
            // the parent [`EntityStore`], ensuring no other modifications can be made to it 
            // as long as this struct lives.
            .expect("expected the entity primary ID to be still valid")
    }

    /// Removes the entity from the store and returns it.
    pub fn remove(self) -> E {
        self.store.remove(&self.target_entity_id)
            // PANIC SAFETY: This can never panic, because this struct mutably borrows from
            // the parent [`EntityStore`], ensuring no other modifications can be made to it 
            // as long as this struct lives.
            .expect("expected the entity primary ID to be still valid")
    }
}

impl<I, E> Debug for OccupiedStoreEntry<'_, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OccupiedStoreEntry<{:?}>",
            self.target_entity_id
        )
    }
}



/// Describes a vacant [`EntityStore`] entry (bound to a specific ID of type `I`).
pub struct VacantStoreEntry<'s, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    store: &'s mut EntityStore<I, E>,

    target_entity_id: I,
}

impl<I, E> VacantStoreEntry<'_, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    pub fn insert(self, entity: E) {
        let insertion_result = self.store.insert_or_replace(self.target_entity_id, entity);

        // PANIC SAFETY: This can never panic, because this struct mutably borrows from
        // the parent [`EntityStore`], ensuring no other modifications can be made to it
        // as long as this struct lives. Because of that the function call above will always
        // return [`EntityStoreInsertionAction::Inserted`], because there is no value to be replaced.
        assert!(
            matches!(
                insertion_result,
                EntityStoreInsertionAction::Inserted
            ),
            "failed to insert value: expected the entity store to not contain this entity"
        );
    }
}

impl<I, E> Debug for VacantStoreEntry<'_, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VacantStoreEntry<{:?}>", self.target_entity_id)
    }
}


/// Describes an entry in [`EntityStore`] based on a given ID.
/// The entry can exist or not (see variants).
#[derive(Debug)]
pub enum EntityStoreEntry<'s, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    Vacant(VacantStoreEntry<'s, I, E>),
    Occupied(OccupiedStoreEntry<'s, I, E>),
}

impl<I, E> EntityStoreEntry<'_, I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    pub fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant(_))
    }

    pub fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied(_))
    }
}



/// Describes the action that was taken when performing an insert-or-update operation on [`EntityStore`].
#[derive(Debug)]
pub enum EntityStoreInsertionAction<E> {
    /// An entity was created.
    Inserted,

    /// An entity was replaced.
    Replaced {
        /// The old entity that is no longer in the store (as it was just replaced).
        old_entity: E,
    },
}



/// An entity store where each entity
/// is uniquely identified (and can be managed) by either:
/// - its primary ID (e.g. [`CategoryId`], which is a UUID newtype), or
/// - its corresponding slot map key (e.g. [`CategoryCacheKey`]).
///
/// When reading the rest of the documentation and method names, keep the following in mind:
/// when we refer to an "ID", we mean the first one (the generic type `I`). When we refer to a "key",
/// we mean the second one (the generic type `K`).
///
/// # Generics
/// The generic `I` is the type of the primary ID of the entity, i.e. no two entities can have the same ID
/// (this invariant is up to the caller to uphold). It must implement [`Eq`]`+`[`Hash`]`+`[`Clone`].
///
/// The generic `K` describes the [`slotmap::Key`]-implementing type that will be used internally
/// to store individual entities in a slotmap (see [`slotmap`] and its [`new_key_type!`] macro).
///
/// Lastly, the generic `E` is the entity - the type that will be stored and accessible based on its primary ID and slotmap key.
///
/// # Implementation details
/// Internally, an entity store is backed by two things:
/// - A bijective map, which is built on top of two [`HashMap`]s, one an inverse of the other,
///   which allows us to obtain a slotmap key for a given entity ID and the inverse.
/// - A [`SlotMap`] for entities, based on the custom entity slot map key - the generic `K`
///   (e.g. [`SlotMap`]`<`[`CategoryCacheKey`]`,`[`CachedCategory`]`>`).
///
///
/// [`new_key_type!`]: slotmap::new_key_type
pub struct EntityStore<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    /// Maps an entity's primary ID (type `I`) to the corresponding entity.
    id_to_entity_map: HashMap<I, E>,
}

impl<I, E> EntityStore<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    /// Initializes a new empty [`EntityStore`].
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            id_to_entity_map: HashMap::new(),
        }
    }


    /// Inserts the provided `entity` into the entity store at the provided `entity_id`,
    /// replacing and returning the old entity under the same ID (if the entity was already in the map).
    pub fn insert_or_replace(&mut self, entity_id: I, entity: E) -> EntityStoreInsertionAction<E> {
        if let Some(existing_entity) = self.id_to_entity_map.get_mut(&entity_id) {
            // The entity already exists in this store, let's replace it.

            let old_entity = std::mem::replace(existing_entity, entity);

            return EntityStoreInsertionAction::Replaced { old_entity };
        }

        let insertion_result = self.id_to_entity_map.insert(entity_id, entity);
        assert!(insertion_result.is_none());

        EntityStoreInsertionAction::Inserted
    }


    /// Returns a standalone entry "view" into the store based on the given `entity_id`.
    ///
    /// The entry is either occupied or vacant, which you can match on, allowing you to
    /// manage the entity under a specific ID in a simpler way (and without need to, in some cases,
    /// awkwardly unwrap things in your function you already know to please the compiler).
    /// The matched variants allow you to perform immutable and mutable operations on the given entry,
    /// depending on whether it already exists or not.
    pub fn entry(&mut self, entity_id: I) -> EntityStoreEntry<'_, I, E> {
        match self.id_to_entity_map.contains_key(&entity_id) {
            true => EntityStoreEntry::Occupied(OccupiedStoreEntry {
                store: self,
                target_entity_id: entity_id,
            }),
            false => EntityStoreEntry::Vacant(VacantStoreEntry {
                store: self,
                target_entity_id: entity_id,
            }),
        }
    }


    /// Returns a reference to an entity based on its ID,
    /// returning `None` if the entity is not present in the store.
    pub fn get<'e>(&'e self, entity_id: &I) -> Option<&'e E> {
        self.id_to_entity_map.get(entity_id)
    }


    /// Returns a mutable reference to an entity based on its ID,
    /// returning `None` if the entity is not present in the store.
    pub fn get_mut<'e>(&'e mut self, entity_id: &I) -> Option<&'e mut E> {
        self.id_to_entity_map.get_mut(entity_id)
    }


    /// Removes the entity from the store based on its ID.
    ///
    /// If the entity was present in the store and has been removed,
    /// this function returns `Some` with the removed entity;
    /// otherwise it returns `None`.
    pub fn remove(&mut self, entity_id: &I) -> Option<E> {
        self.id_to_entity_map.remove(entity_id)
    }

    pub fn contains(&self, entity_id: &I) -> bool {
        self.id_to_entity_map.contains_key(entity_id)
    }

    pub fn clear(&mut self) {
        self.id_to_entity_map.clear();
    }

    pub fn iter(&self) -> hash_map::Iter<'_, I, E> {
        self.id_to_entity_map.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<'_, I, E> {
        self.id_to_entity_map.iter_mut()
    }
}

impl<I, E> IntoIterator for EntityStore<I, E>
where
    I: Eq + Hash + Clone + Debug,
{
    type IntoIter = hash_map::IntoIter<I, E>;
    type Item = (I, E);

    fn into_iter(self) -> Self::IntoIter {
        self.id_to_entity_map.into_iter()
    }
}




#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
    struct SampleEntityId(u64);

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct SampleEntity {
        value: u64,
    }


    #[test]
    fn store_insertion_and_removal_works() {
        let mut store = EntityStore::<SampleEntityId, SampleEntity>::new();

        let entity_id = SampleEntityId(1);
        let entity_first = SampleEntity { value: 2 };
        let entity_second = SampleEntity { value: 1 };


        assert!(store.entry(entity_id).is_vacant());

        assert!(store.get(&entity_id).is_none());
        assert!(store.get_mut(&entity_id).is_none());



        let EntityStoreInsertionAction::Inserted =
            store.insert_or_replace(entity_id, entity_first.clone())
        else {
            panic!();
        };

        let EntityStoreInsertionAction::Replaced { old_entity } =
            store.insert_or_replace(entity_id, entity_second.clone())
        else {
            panic!();
        };

        assert_eq!(old_entity, entity_first);

        assert_eq!(store.get(&entity_id).unwrap(), &entity_second);
        assert_eq!(store.get_mut(&entity_id).unwrap(), &entity_second);


        let EntityStoreEntry::Occupied(occupied_entry) = store.entry(entity_id) else {
            panic!();
        };

        assert_eq!(occupied_entry.get(), &entity_second);

        assert_eq!(store.remove(&entity_id).unwrap(), entity_second);
        assert!(store.remove(&entity_id).is_none());

        assert!(store.entry(entity_id).is_vacant());

        assert!(store.get(&entity_id).is_none());
        assert!(store.get_mut(&entity_id).is_none());
    }
}
