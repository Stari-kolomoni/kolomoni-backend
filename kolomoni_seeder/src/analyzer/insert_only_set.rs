use std::{collections::HashMap, hash::Hash, sync::Arc};

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};


pub trait InternalId {
    type InternalId: PartialEq + Eq + Clone + Hash;

    fn internal_id(&self) -> Self::InternalId;
}



#[derive(Debug)]
pub struct GrowingKeyedSet<V>
where
    V: InternalId,
{
    map: Arc<RwLock<HashMap<V::InternalId, V>>>,
}

impl<V> GrowingKeyedSet<V>
where
    V: InternalId,
{
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, value: V) -> Result<(), V> {
        let key = value.internal_id();


        {
            let mut write_locked_map = self.map.write();
            if write_locked_map.contains_key(&key) {
                return Err(value);
            }

            write_locked_map.insert(key.clone(), value);
        }

        Ok(())
    }

    pub fn insert_or_replace(&self, value: V) {
        let key = value.internal_id();

        {
            let mut write_locked_map = self.map.write();
            write_locked_map.insert(key.clone(), value);
        }
    }

    pub fn get(&self, key: &V::InternalId) -> Option<MappedRwLockReadGuard<'_, V>> {
        let read_locked_map = self.map.read();

        let mapped_value_access = RwLockReadGuard::try_map(read_locked_map, |map| map.get(key));

        match mapped_value_access {
            Ok(inner_value_ref) => Some(inner_value_ref),
            Err(_) => None,
        }
    }

    pub fn read_inner(&self) -> RwLockReadGuard<'_, HashMap<V::InternalId, V>> {
        self.map.read()
    }
}

impl<V> Clone for GrowingKeyedSet<V>
where
    V: InternalId,
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
