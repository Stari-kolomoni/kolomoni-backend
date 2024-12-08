use std::{
    collections::{hash_map, HashMap},
    hash::Hash,
};


pub trait InternalId {
    type InternalId: PartialEq + Eq + Clone + Hash;

    fn internal_id(&self) -> Self::InternalId;
}



#[derive(Debug)]
pub struct GrowingKeyedSet<V>
where
    V: InternalId,
{
    map: HashMap<V::InternalId, V>,
}

impl<V> GrowingKeyedSet<V>
where
    V: InternalId,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: V) -> Result<(), V> {
        let key = value.internal_id();

        if self.map.contains_key(&key) {
            return Err(value);
        }

        self.map.insert(key.clone(), value);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn insert_or_replace(&mut self, value: V) {
        let key = value.internal_id();

        self.map.insert(key.clone(), value);
    }

    pub fn get(&self, key: &V::InternalId) -> Option<&V> {
        self.map.get(key)
    }

    pub fn values(&self) -> hash_map::Values<'_, V::InternalId, V> {
        self.map.values()
    }
}

impl<V> Clone for GrowingKeyedSet<V>
where
    V: InternalId + Clone,
{
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}


/*
#[derive(Debug, Clone)]
pub struct InsertOnlyKeyedDashSetHandle<V>
where
    V: Key,
{
    keyed_dash_set: InsertOnlyKeyedDashSet<V>,
    key: V::Key,
}

impl<V> InsertOnlyKeyedDashSetHandle<V>
where
    V: Key,
{
    #[inline]
    fn new(keyed_dash_set: InsertOnlyKeyedDashSet<V>, key: V::Key) -> Self {
        Self {
            keyed_dash_set,
            key,
        }
    }

    fn value(&self) -> Option<MappedRwLockReadGuard<'_, V>> {
        self.keyed_dash_set.get(&self.key)
    }
}
 */
