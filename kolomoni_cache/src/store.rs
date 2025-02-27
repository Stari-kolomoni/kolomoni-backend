use std::{collections::HashMap, fmt::Debug, hash::Hash};

use slotmap::SlotMap;


/// Describes an occupied [`EntityStore`] entry.
pub struct OccupiedStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    store: &'s mut EntityStore<I, K, E>,

    target_entity_id: I,
    target_slotmap_key: K,
}

impl<'s, I, K, E> OccupiedStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    /// Returns the slotmap key that belongs to this entity.
    pub fn key(&self) -> K {
        self.target_slotmap_key
    }

    /// Returns a reference to the entity.
    pub fn get(&self) -> &E {
        self.store.entity_slot_map.get(self.target_slotmap_key)
            // PANIC SAFETY: This can never panic, because this struct mutably borrows from
            // the parent [`EntityStore`], ensuring no other modifications can be made to it 
            // as long as this struct lives.
            .expect("expected the entity SlotMap key to be still valid")
    }

    /// Returns a mutable reference to the entity.
    pub fn get_mut(&mut self) -> &mut E {
        self.store.entity_slot_map.get_mut(self.target_slotmap_key)
            // PANIC SAFETY: This can never panic, because this struct mutably borrows from
            // the parent [`EntityStore`], ensuring no other modifications can be made to it 
            // as long as this struct lives.
            .expect("expected the entity SlotMap key to be still valid")
    }

    /// Removes the entity from the store and returns it.
    pub fn remove(self) -> E {
        self.store.remove_with_id_and_key(&self.target_entity_id, self.target_slotmap_key)
            // PANIC SAFETY: This can never panic, because this struct mutably borrows from
            // the parent [`EntityStore`], ensuring no other modifications can be made to it 
            // as long as this struct lives.
            .expect("expected the entity SlotMap key to be still valid")
    }
}

impl<'s, I, K, E> Debug for OccupiedStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
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
pub struct VacantStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    store: &'s mut EntityStore<I, K, E>,

    target_entity_id: I,
}

impl<'s, I, K, E> VacantStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    pub fn insert(self, entity: E) -> K {
        let insertion_result = self.store.insert_or_replace(self.target_entity_id, entity);

        let EntityStoreInsertionAction::Inserted { entity_slotmap_key } = insertion_result else {
            // PANIC SAFETY: This can never panic, because this struct mutably borrows from
            // the parent [`EntityStore`], ensuring no other modifications can be made to it
            // as long as this struct lives. Because of that the function call above will always
            // return [`EntityStoreInsertionAction::Inserted`], because there is no value to be replaced.
            panic!(
                "failed to properly insert value: expected the SlotMap to not contain this entity"
            );
        };

        entity_slotmap_key
    }
}

impl<'s, I, K, E> Debug for VacantStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VacantStoreEntry<{:?}>", self.target_entity_id)
    }
}


/// Describes an entry in [`EntityStore`] based on a given ID.
/// The entry can exist or not (see variants).
#[derive(Debug)]
pub enum EntityStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    Vacant(VacantStoreEntry<'s, I, K, E>),
    Occupied(OccupiedStoreEntry<'s, I, K, E>),
}

impl<'s, I, K, E> EntityStoreEntry<'s, I, K, E>
where
    K: slotmap::Key,
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
pub enum EntityStoreInsertionAction<K, E>
where
    K: slotmap::Key,
{
    /// An entity was created.
    Inserted {
        /// The slotmap key this entity now lives under.
        entity_slotmap_key: K,
    },

    /// An entity was replaced.
    Replaced {
        /// The slotmap key this entity lives under.
        entity_slotmap_key: K,

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
pub struct EntityStore<I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    /// Maps an entity's primary ID (type `I`) to its corresponding slotmap key (type `K`).
    id_to_slotmap_key_map: HashMap<I, K>,

    /// Maps an entity's slotmap key (type `K`) to its corresponding primary ID (type `I`).
    slotmap_key_to_id_map: HashMap<K, I>,

    /// Stores entities (type `E`), indexed by slotmap keys (type `K`).
    entity_slot_map: SlotMap<K, E>,
}

impl<I, K, E> EntityStore<I, K, E>
where
    K: slotmap::Key,
    I: Eq + Hash + Clone + Debug,
{
    /// Initializes a new empty [`EntityStore`].
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            id_to_slotmap_key_map: HashMap::new(),
            slotmap_key_to_id_map: HashMap::new(),
            entity_slot_map: SlotMap::with_key(),
        }
    }

    /// Inserts the provided `entity_id` and `entity_slotmap_key` pair into the internal bijective map,
    /// allowing us to look up entities by either their IDs or slotmap keys.
    ///
    /// *This is an internal method.*
    ///
    ///
    /// # Panics
    /// Panics if `entity_id` or `entity_slotmap_key` are already present in their respective parts of the bijective map.
    ///
    /// It is therefore up to the caller to ensure that this method is called as appropriate.
    /// This method is private, so if it panics, it indicates the [`EntityStore`] contains a bug in
    /// how it manages its bijective map.
    fn insert_into_bijective_map(&mut self, entity_id: I, entity_slotmap_key: K) {
        let old_slotmap_key = self
            .id_to_slotmap_key_map
            .insert(entity_id.clone(), entity_slotmap_key);

        // PANIC SAFETY: If the internal invariants of [`EntityStore`] are upheld
        // (i.e. if the bijective map is properly managed), this will never panic.
        // If this panics, it indicates the bijective map is mismanaged, which is a bug.
        assert!(old_slotmap_key.is_none());


        let old_entity_id = self
            .slotmap_key_to_id_map
            .insert(entity_slotmap_key, entity_id);

        // PANIC SAFETY: If the internal invariants of [`EntityStore`] are upheld
        // (i.e. if the bijective map is properly managed), this will never panic.
        // If this panics, it indicates the bijective map is mismanaged, which is a bug.
        assert!(old_entity_id.is_none());
    }

    /// Removes the provided `entity_id` and `entity_slotmap_key` pair from the internal bijective map.
    ///
    /// *This is an internal method.*
    ///
    ///
    /// # Panics
    /// Panics if `entity_id` or `entity_slotmap_key` are not present in their respective parts of the bijective map,
    /// or if the ID and key don't belong to the same entity.
    ///
    /// It is therefore up to the caller to ensure that this method is called as appropriate.
    /// This method is private, so if it panics, it indicates the [`EntityStore`] contains a bug in
    /// how it manages its bijective map.
    fn remove_from_bijective_map(&mut self, entity_id: &I, entity_slotmap_key: &K) {
        let old_slotmap_key = self.id_to_slotmap_key_map.remove(entity_id);

        // PANIC SAFETY: If the internal invariants of [`EntityStore`] are upheld
        // (i.e. if the bijective map is properly managed), this will never panic.
        // If this panics, it indicates the bijective map is mismanaged, which is a bug.
        assert_eq!(&old_slotmap_key.unwrap(), entity_slotmap_key);


        let old_entity_id = self.slotmap_key_to_id_map.remove(entity_slotmap_key);

        // PANIC SAFETY: If the internal invariants of [`EntityStore`] are upheld
        // (i.e. if the bijective map is properly managed), this will never panic.
        // If this panics, it indicates the bijective map is mismanaged, which is a bug.
        assert_eq!(&old_entity_id.unwrap(), entity_id);
    }

    /// Returns an optional reference to an entity's primary ID based on its slotmap key.
    #[inline]
    pub fn get_id_by_key(&self, entity_slotmap_key: K) -> Option<&I> {
        self.slotmap_key_to_id_map.get(&entity_slotmap_key)
    }

    /// Inserts the provided `entity` into the entity store at the provided `entity_id`,
    /// replacing and returning the old entity under the same ID (if the entity was already in the map).
    pub fn insert_or_replace(
        &mut self,
        entity_id: I,
        entity: E,
    ) -> EntityStoreInsertionAction<K, E> {
        if let Some(existing_entity_slotmap_key) = self.id_to_slotmap_key_map.get(&entity_id) {
            // The entity already exists in this store, let's replace it.

            let current_entity_mut = self.entity_slot_map.get_mut(*existing_entity_slotmap_key)
                // PANIC SAFETY: If the internal bijective map and slotmap are in sync, 
                // this will never panic due to the parent if statement. If they happen to not
                // be in sync, we consider this a mismanagement bug and this will catch it.
                .expect(
                    "failed to get entity: \
                    not in the SlotMap, but present in the ID-to-key HashMap"
                );

            let old_entity = std::mem::replace(current_entity_mut, entity);

            return EntityStoreInsertionAction::Replaced {
                entity_slotmap_key: *existing_entity_slotmap_key,
                old_entity,
            };
        }


        // The entity is not present in this store, let's insert it.
        let new_entity_slotmap_key = self.entity_slot_map.insert(entity);

        self.insert_into_bijective_map(entity_id, new_entity_slotmap_key);


        EntityStoreInsertionAction::Inserted {
            entity_slotmap_key: new_entity_slotmap_key,
        }
    }

    /// Returns a standalone entry "view" into the store based on the given `entity_id`.
    ///
    /// The entry is either occupied or vacant, which you can match on, allowing you to
    /// manage the entity under a specific ID in a simpler way (and without need to, in some cases,
    /// awkwardly unwrap things in your function you already know to please the compiler).
    /// The matched variants allow you to perform immutable and mutable operations on the given entry,
    /// depending on whether it already exists or not.
    pub fn entry(&mut self, entity_id: I) -> EntityStoreEntry<'_, I, K, E> {
        match self.id_to_slotmap_key_map.get(&entity_id).copied() {
            Some(existing_slotmap_key) => EntityStoreEntry::Occupied(OccupiedStoreEntry {
                store: self,
                target_entity_id: entity_id,
                target_slotmap_key: existing_slotmap_key,
            }),
            None => EntityStoreEntry::Vacant(VacantStoreEntry {
                store: self,
                target_entity_id: entity_id,
            }),
        }
    }

    /// Returns a reference to an entity based on its ID,
    /// returning `None` if the entity is not present in the store.
    pub fn get<'e>(&'e self, entity_id: &I) -> Option<&'e E> {
        let entity_slotmap_key = self.id_to_slotmap_key_map.get(entity_id)?;

        let Some(borrowed_entity) = self.entity_slot_map.get(*entity_slotmap_key) else {
            // PANIC SAFETY: If the internal bijective map and slotmap are in sync,
            // this will never panic due to the `get(...)?` call at the top of the method.
            // If they happen to not be in sync, we consider this a mismanagement bug
            // and this will catch it.
            panic!(
                "failed to retrieve entity from store: \
                is present in the ID-to-key HashMap, but not in the SlotMap",
            );
        };

        Some(borrowed_entity)
    }

    /// Returns a reference to an entity based on its key,
    /// returning `None` if the entity is not present in the store.
    #[inline]
    pub fn get_by_key(&self, entity_slotmap_key: K) -> Option<&E> {
        self.entity_slot_map.get(entity_slotmap_key)
    }

    /// Returns a mutable reference to an entity based on its ID,
    /// returning `None` if the entity is not present in the store.
    pub fn get_mut<'e>(&'e mut self, entity_id: &I) -> Option<&'e mut E> {
        let entity_slotmap_key = self.id_to_slotmap_key_map.get(entity_id)?;

        let Some(borrowed_entity) = self.entity_slot_map.get_mut(*entity_slotmap_key) else {
            // PANIC SAFETY: If the internal bijective map and slotmap are in sync,
            // this will never panic due to the `get(...)?` call at the top of the method.
            // If they happen to not be in sync, we consider this a mismanagement bug
            // and this will catch it.
            panic!(
                "failed to retrieve entity from store: \
                present in the ID-to-key HashMap, but not in the SlotMap",
            );
        };

        Some(borrowed_entity)
    }


    /// Returns an optional tuple containing a mutable reference to an entity and
    /// its corresponding slotmap key. This method returns `None` if the entity
    /// is not present in the store.
    pub fn get_mut_and_key<'e>(&'e mut self, entity_id: &I) -> Option<(&'e mut E, K)> {
        let entity_slotmap_key = self.id_to_slotmap_key_map.get(entity_id)?;

        let Some(borrowed_entity) = self.entity_slot_map.get_mut(*entity_slotmap_key) else {
            // PANIC SAFETY: If the internal bijective map and slotmap are in sync,
            // this will never panic due to the `get(...)?` call at the top of the method.
            // If they happen to not be in sync, we consider this a mismanagement bug
            // and this will catch it.
            panic!(
                "failed to retrieve entity from store: \
                is present in the primary-ID-to-key HashMap, but not in the SlotMap",
            );
        };

        Some((borrowed_entity, *entity_slotmap_key))
    }


    /// Returns a mutable reference to an entity based on its key,
    /// returning `None` if the entity is not present in the store.
    #[inline]
    pub fn get_mut_by_key(&mut self, entity_slotmap_key: K) -> Option<&mut E> {
        self.entity_slot_map.get_mut(entity_slotmap_key)
    }

    /// Returns an entity's slot map key based on its ID,
    /// returning `None` if the entity is not present in the store.
    #[inline]
    pub fn get_key(&self, entity_id: &I) -> Option<K> {
        self.id_to_slotmap_key_map.get(entity_id).copied()
    }


    /// Removes the entity from the store based on its ID.
    ///
    /// If the entity was present in the store and has been removed,
    /// this function returns `Some` with the removed entity;
    /// otherwise it returns `None`.
    pub fn remove(&mut self, entity_id: &I) -> Option<E> {
        let entity_slotmap_key = *self.id_to_slotmap_key_map.get(entity_id)?;


        let Some(removed_entity) = self.entity_slot_map.remove(entity_slotmap_key) else {
            // PANIC SAFETY: If `self.id_to_slotmap_key_map` and `self.entity_slot_map` are in sync,
            // this will never panic due to the `get(...)?` call above.
            // If they happen to not be in sync, this is considered a bug and this will catch it.
            panic!(
                "failed to remove entity: \
                present in the primary-ID-to-key HashMap, but missing from the SlotMap"
            );
        };

        self.remove_from_bijective_map(entity_id, &entity_slotmap_key);


        Some(removed_entity)
    }

    /// Removes the entity from the store based on its key.
    ///
    /// If the entity was present in the store and has been removed,
    /// this function returns `Some` with the removed entity;
    /// otherwise it returns `None`.
    pub fn remove_by_key(&mut self, entity_slotmap_key: K) -> Option<E> {
        let entity_id = self
            .slotmap_key_to_id_map
            .get(&entity_slotmap_key)?
            .to_owned();


        let Some(removed_entity) = self.entity_slot_map.remove(entity_slotmap_key) else {
            // PANIC SAFETY: If `self.id_to_slotmap_key_map` and `self.entity_slot_map` are in sync,
            // this will never panic due to the `get(...)?` call above.
            // If they happen to not be in sync, this is considered a bug and this will catch it.
            panic!(
                "failed to remove entity: \
                present in the primary-ID-to-key HashMap, but missing from the SlotMap"
            );
        };

        self.remove_from_bijective_map(&entity_id, &entity_slotmap_key);


        Some(removed_entity)
    }

    /// Removes the entity from the store using its associated entity ID
    /// and key at the same time.
    ///
    /// If the entity was present in the store and has just been removed,
    /// this function returns `Some` with the removed entity;
    /// otherwise it returns `None`.
    ///
    ///
    /// # Invariants
    /// It is up to the caller to ensure `entity_id` and `entity_slotmap_key` belong to the same entity
    /// and that they exist. If either or those things aren't true, this method will panic.
    fn remove_with_id_and_key(&mut self, entity_id: &I, entity_slotmap_key: K) -> Option<E> {
        let removed_entity = self.entity_slot_map.remove(entity_slotmap_key)?;

        self.remove_from_bijective_map(entity_id, &entity_slotmap_key);

        Some(removed_entity)
    }

    pub fn clear(&mut self) {
        self.id_to_slotmap_key_map.clear();
        self.slotmap_key_to_id_map.clear();
        self.entity_slot_map.clear();
    }
}



#[cfg(test)]
mod test {
    use super::*;


    slotmap::new_key_type! { struct SampleKey; }

    #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
    struct SampleEntityId(u64);

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct SampleEntity {
        value: u64,
    }


    #[test]
    fn store_insertion_and_removal_works() {
        let mut store = EntityStore::<SampleEntityId, SampleKey, SampleEntity>::new();

        let entity_id = SampleEntityId(1);
        let entity_first = SampleEntity { value: 2 };
        let mut entity = SampleEntity { value: 1 };


        assert!(store.entry(entity_id).is_vacant());

        assert!(store.get(&entity_id).is_none());
        assert!(store.get_mut(&entity_id).is_none());
        assert!(store.get_key(&entity_id).is_none());
        assert!(store.get_mut_and_key(&entity_id).is_none());



        let EntityStoreInsertionAction::Inserted {
            entity_slotmap_key: entity_one_key_first,
        } = store.insert_or_replace(entity_id, entity_first.clone())
        else {
            panic!();
        };

        let EntityStoreInsertionAction::Replaced {
            entity_slotmap_key: entity_one_key,
            old_entity,
        } = store.insert_or_replace(entity_id, entity.clone())
        else {
            panic!();
        };

        assert_eq!(entity_one_key, entity_one_key_first);
        assert_eq!(old_entity, entity_first);


        assert_eq!(store.get_key(&entity_id).unwrap(), entity_one_key);
        assert_eq!(store.get(&entity_id).unwrap(), &entity);

        assert_eq!(store.get_mut(&entity_id).unwrap(), &entity);
        assert_eq!(
            store.get_mut_by_key(entity_one_key).unwrap(),
            &entity
        );

        assert_eq!(
            store.get_mut_and_key(&entity_id).unwrap(),
            (&mut entity, entity_one_key)
        );

        assert_eq!(store.get_by_key(entity_one_key).unwrap(), &entity);
        assert_eq!(
            store.get_id_by_key(entity_one_key).unwrap(),
            &entity_id
        );


        let EntityStoreEntry::Occupied(occupied_entry) = store.entry(entity_id) else {
            panic!();
        };

        assert_eq!(occupied_entry.key(), entity_one_key);
        assert_eq!(occupied_entry.get(), &entity);


        assert_eq!(store.remove(&entity_id).unwrap(), entity);
        assert!(store.remove(&entity_id).is_none());


        assert!(store.entry(entity_id).is_vacant());

        assert!(store.get(&entity_id).is_none());
        assert!(store.get_mut(&entity_id).is_none());
        assert!(store.get_key(&entity_id).is_none());
        assert!(store.get_mut_and_key(&entity_id).is_none());


        let EntityStoreInsertionAction::Inserted {
            entity_slotmap_key: entity_one_key,
        } = store.insert_or_replace(entity_id, entity.clone())
        else {
            panic!();
        };


        assert!(store.get(&entity_id).is_some());


        assert_eq!(
            store.remove_by_key(entity_one_key).unwrap(),
            entity
        );
        assert!(store.remove_by_key(entity_one_key).is_none());

        assert!(store.entry(entity_id).is_vacant());

        assert!(store.get(&entity_id).is_none());
        assert!(store.get_mut(&entity_id).is_none());
        assert!(store.get_key(&entity_id).is_none());
        assert!(store.get_mut_and_key(&entity_id).is_none());
    }
}
