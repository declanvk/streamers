use std::mem;
use std::borrow::Borrow;
use std::fmt;

use super::key::{KeyProbe, TreeKey};
use super::node::{RadixNode, recursive_insert, recursive_find, recursive_mut_find, recursive_remove};
use super::entry::KeyValue;

#[derive(Debug)]
pub struct RadixTree<K: TreeKey, V> {
    size: usize,
    root: Option<Box<RadixNode<K, V>>>,
}

impl<K: TreeKey, V> RadixTree<K, V> {
    pub fn new() -> Self {
        RadixTree {
            size: 0,
            root: None,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: TreeKey + AsRef<[u8]>,
    {
        self.get(key).is_some()
    }

    pub fn get<'k, 'v, Q: ?Sized>(&'v self, key: &'k Q) -> Option<&'v V>
    where
        K: Borrow<Q>,
        Q: TreeKey + AsRef<[u8]>,
    {
        if self.root.is_some() {
            let probe = KeyProbe::new(&key);

            recursive_find(self.root.as_ref().unwrap(), probe)
        } else {
            None
        }
    }

    pub fn get_mut<'k, 'v, Q: ?Sized>(&'v mut self, key: &'k Q) -> Option<&'v mut V>
    where
        K: Borrow<Q>,
        Q: TreeKey + AsRef<[u8]>,
    {
        if self.root.is_some() {
            let probe = KeyProbe::new(&key);
            
            recursive_mut_find(self.root.as_mut().unwrap(), probe)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let old_entry = if self.root.is_some() {
            let old_root = mem::replace(&mut self.root, None).unwrap();

            let probe = KeyProbe::new(&key);
            let new_entry = KeyValue::new(key.clone(), value);
            let (updated_node, old_entry) = recursive_insert(old_root, probe, new_entry);

            let _ = mem::replace(&mut self.root, Some(updated_node));

            old_entry
        } else {
            let new_leaf = RadixNode::new_leaf(key, value);
            self.root = Some(box new_leaf);
            None
        };

        if old_entry.is_none() {
            self.size += 1;
        }

        old_entry
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: TreeKey,
    {
        let old_entry = if self.root.is_some() {
            let probe = KeyProbe::new(key);
            let old_root = mem::replace(&mut self.root, None).unwrap();

            let (updated_node, old_entry) = recursive_remove(old_root, probe);

            let _ = mem::replace(&mut self.root, updated_node);

            old_entry
        } else {
            None
        };

        if old_entry.is_some() {
            self.size -= 1;
        }

        old_entry
    }
}

#[cfg(any(debug_assertions, test))]
use super::node::debug::TreeView;

#[cfg(any(debug_assertions, test))]
impl<K, V> RadixTree<K, V> where K: TreeKey, V: fmt::Debug {
    pub fn debug_view<'a>(&'a self) -> TreeView<'a, K, V> {
        TreeView::new(self.root.as_ref().expect("Tried to view an empty tree!"), 7)
    }
}

#[cfg(test)]
mod tree_tests {
    use super::*;

    #[test]
    fn create_tree() {
        let rax = RadixTree::<String, ()>::new();

        assert!(rax.is_empty());
    }

    #[test]
    fn insert_non_overlapping() {
        let mut rax = RadixTree::<&str, ()>::new();

        rax.insert("hello", ());
        rax.insert("goodbye", ());

        assert!(!rax.is_empty());
        assert_eq!(rax.len(), 2);
    }

    #[test]
    fn insert_overlapping() {
        let mut rax = RadixTree::<&str, ()>::new();

        rax.insert("hello", ());
        rax.insert("hella", ());

        assert!(!rax.is_empty());
        assert_eq!(rax.len(), 2);
    }

    #[test]
    fn insert_retrieve() {
        let mut rax = RadixTree::<&str, usize>::new();

        rax.insert("hello", 1);

        assert!(!rax.is_empty());
        assert_eq!(rax.len(), 1);

        let value = rax.get(&"hello");

        assert!(value.is_some());
        assert_eq!(value.unwrap(), &1);
    }

    #[test]
    fn retrieve_nonexistent() {
        let mut rax = RadixTree::<&str, usize>::new();

        rax.insert("hello", 1);

        assert!(!rax.is_empty());
        assert_eq!(rax.len(), 1);

        let value = rax.get(&"goodbye");

        assert!(value.is_none());
    }

    #[test]
    fn retrieve_overlapping() {
        let mut rax = RadixTree::<&str, usize>::new();

        rax.insert("hello", 1);
        rax.insert("hel", 2);

        assert!(!rax.is_empty());
        assert_eq!(rax.len(), 2);

        let value = rax.get(&"hel");

        assert!(value.is_some());
        assert_eq!(value.unwrap(), &2);
    }

    #[test]
    fn insert_retrieve_mutate() {
        let mut rax = RadixTree::<&str, usize>::new();

        rax.insert("hello", 1);

        assert!(!rax.is_empty());
        assert_eq!(rax.len(), 1);

        {
            let value: Option<&mut usize> = rax.get_mut(&"hello");
            assert!(value.is_some());

            *value.unwrap() += 4;
        }

        let value = rax.get(&"hello");

        assert!(value.is_some());
        assert_eq!(value.unwrap(), &5);
    }
}


