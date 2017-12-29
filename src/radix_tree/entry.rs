use std::mem;
use std::fmt;
use super::key::TreeKey;

#[derive(Clone, PartialEq, Eq)]
pub struct KeyValue<K: TreeKey, V> {
    key: K,
    value: V,
}

impl<K: TreeKey, V> KeyValue<K, V> {
    pub fn new(key: K, value: V) -> Self {
        KeyValue { key, value }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn key_mut(&mut self) -> &mut K {
        &mut self.key
    }

    pub fn take_key(self) -> K {
        self.key
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    pub fn take_value(self) -> V {
        self.value
    }

    pub fn swap_value(&mut self, mut new_value: V) -> V {
        mem::swap(&mut self.value, &mut new_value);

        new_value
    }
}

impl<K: TreeKey + fmt::Debug, V: fmt::Debug> fmt::Debug for KeyValue<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KeyValue {{ key: {:?}, value: {:?}}}", self.key, self.value)
    }
}
