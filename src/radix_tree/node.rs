use std::mem;
use std::fmt;
use std::slice;
use std::iter;

use super::key::{KeyMatchResult, KeyPrefix, KeyProbe, TreeKey};
use super::entry::KeyValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeChildren<K: TreeKey, V> {
    children: Vec<(u8, Box<RadixNode<K, V>>)>,
    empty_child: Option<Box<RadixNode<K, V>>>,
}

impl<K: TreeKey, V> NodeChildren<K, V> {
    pub fn new() -> Self {
        NodeChildren {
            children: Vec::new(),
            empty_child: None,
        }
    }

    pub fn contains_child(&self, prefix: u8) -> bool {
        let search_result = self.children
            .binary_search_by(|&(ref value, _)| value.cmp(&prefix));

        match search_result {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn contains_empty(&self) -> bool {
        self.empty_child.is_some()
    }

    pub fn get_child(&self, possible_prefix: Option<u8>) -> Option<&Box<RadixNode<K, V>>> {
        if let Some(prefix) = possible_prefix {
            let search_result = self.children
                .binary_search_by(|&(ref value, _)| value.cmp(&prefix));

            match search_result {
                Ok(found_index) => {
                    let (_, ref child) = self.children[found_index];
                    Some(child)
                }
                Err(_) => None,
            }
        } else {
            if let Some(ref child) = self.empty_child {
                Some(child)
            } else {
                None
            }
        }
    }

    pub fn get_child_mut(
        &mut self,
        possible_prefix: Option<u8>,
    ) -> Option<&mut Box<RadixNode<K, V>>> {
        if let Some(prefix) = possible_prefix {
            let search_result = self.children
                .binary_search_by(|&(ref value, _)| value.cmp(&prefix));

            match search_result {
                Ok(found_index) => {
                    let (_, ref mut child) = self.children[found_index];
                    Some(child)
                }
                Err(_) => None,
            }
        } else {
            if let Some(ref mut child) = self.empty_child {
                Some(child)
            } else {
                None
            }
        }
    }

    pub fn insert_child(
        &mut self,
        possible_prefix: Option<u8>,
        new_child: Box<RadixNode<K, V>>,
    ) -> Option<Box<RadixNode<K, V>>> {
        if let Some(prefix) = possible_prefix {
            let search_result = self.children
                .binary_search_by(|&(ref value, _)| value.cmp(&prefix));

            match search_result {
                Ok(found_index) => {
                    let (_, ref mut old_child) = self.children[found_index];
                    let old_child = mem::replace(old_child, new_child);
                    Some(old_child)
                }
                Err(insert_index) => {
                    self.children.insert(insert_index, (prefix, new_child));
                    None
                }
            }
        } else {
            if self.empty_child.is_some() {
                let old_child = mem::replace(
                    self.empty_child
                        .as_mut()
                        .expect(&format!("{}: {}", file!(), line!())),
                    new_child,
                );
                Some(old_child)
            } else {
                self.empty_child = Some(new_child);
                None
            }
        }
    }

    pub fn remove_child(&mut self, possible_prefix: Option<u8>) -> Option<Box<RadixNode<K, V>>> {
        if let Some(prefix) = possible_prefix {
            let search_result = self.children
                .binary_search_by(|&(ref value, _)| value.cmp(&prefix));

            match search_result {
                Ok(found_index) => {
                    let (_, child) = self.children.remove(found_index);
                    Some(child)
                }
                Err(_) => None,
            }
        } else {
            if self.empty_child.is_some() {
                let old_child = mem::replace(&mut self.empty_child, None).expect(&format!(
                    "{}: {}",
                    file!(),
                    line!()
                ));

                Some(old_child)
            } else {
                None
            }
        }
    }

    pub fn iter<'a>(&'a self) -> ChildrenIter<'a, K, V>
    where
        K: 'a + TreeKey,
        V: 'a,
    {
        ChildrenIter {
            iter: self.children.iter(),
        }
    }
}

pub struct ChildrenIter<'a, K: 'a, V: 'a>
where
    K: TreeKey,
{
    iter: slice::Iter<'a, (u8, Box<RadixNode<K, V>>)>,
}

impl<'a, K: 'a, V: 'a> iter::Iterator for ChildrenIter<'a, K, V>
where
    K: TreeKey,
{
    type Item = &'a (u8, Box<RadixNode<K, V>>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadixInteriorNode<K: TreeKey, V> {
    prefix: KeyPrefix,
    children: NodeChildren<K, V>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct RadixLeafNode<K: TreeKey, V> {
    entry: Box<KeyValue<K, V>>,
    remaining_key: KeyPrefix,
}

impl<K: TreeKey + fmt::Debug, V: fmt::Debug> fmt::Debug for RadixLeafNode<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{:?}> -> {:?}", self.remaining_key, self.entry)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadixNode<K: TreeKey, V> {
    // Leaf nodes will have no children and the data value will be set.
    // Leaf nodes can be direct descendents of only branch nodes in the
    // scenario that there is no more key bytes after the branch byte
    Leaf(RadixLeafNode<K, V>),
    // Branch nodes occur in the interior of the tree.
    // They will contain > 1 children, and the key ptr will point to
    // an array of byte values that will prefix each child as part of the key
    Interior(RadixInteriorNode<K, V>),
}

impl<K: TreeKey, V> RadixNode<K, V> {
    pub fn new_leaf(key: K, value: V) -> Self {
        let key_bytes = key.as_bytes();
        RadixNode::Leaf(RadixLeafNode {
            remaining_key: KeyPrefix::new(key_bytes),
            entry: box KeyValue::new(key.clone(), value),
        })
    }

    pub fn is_leaf(&self) -> bool {
        match *self {
            RadixNode::Leaf(_) => true,
            _ => false,
        }
    }

    pub fn get_leaf(&self) -> &RadixLeafNode<K, V> {
        match *self {
            RadixNode::Leaf(ref node) => node,
            _ => panic!("called `RadixNode::get_leaf()` on a `Interior` node"),
        }
    }

    pub fn get_leaf_mut(&mut self) -> &mut RadixLeafNode<K, V> {
        match *self {
            RadixNode::Leaf(ref mut node) => node,
            _ => panic!("called `RadixNode::get_leaf()` on a `Interior` node"),
        }
    }

    pub fn unwrap_leaf(self) -> RadixLeafNode<K, V> {
        match self {
            RadixNode::Leaf(node) => node,
            _ => panic!("called `RadixNode::unwrap_leaf()` on a `Interior` node"),
        }
    }

    pub fn is_interior(&self) -> bool {
        match *self {
            RadixNode::Interior(_) => true,
            _ => false,
        }
    }

    pub fn get_interior(&self) -> &RadixInteriorNode<K, V> {
        match *self {
            RadixNode::Interior(ref node) => node,
            _ => panic!("called `RadixNode::get_interior()` on a `Leaf` node"),
        }
    }

    pub fn get_interior_mut(&mut self) -> &mut RadixInteriorNode<K, V> {
        match *self {
            RadixNode::Interior(ref mut node) => node,
            _ => panic!("called `RadixNode::get_interior()` on a `Leaf` node"),
        }
    }

    pub fn unwrap_interior(self) -> RadixInteriorNode<K, V> {
        match self {
            RadixNode::Interior(node) => node,
            _ => panic!("called `RadixNode::unwrap_interior()` on a `Leaf` node"),
        }
    }
}

pub fn recursive_insert<'a, K: TreeKey, V>(
    current: Box<RadixNode<K, V>>,
    probe: KeyProbe<'a>,
    new_entry: KeyValue<K, V>,
) -> (Box<RadixNode<K, V>>, Option<V>) {
    match *current {
        RadixNode::Leaf(mut node) => match node.remaining_key.match_with(probe) {
            KeyMatchResult::Complete => {
                let old_value = node.entry.swap_value(new_entry.take_value());

                (box RadixNode::Leaf(node), Some(old_value))
            }
            KeyMatchResult::Partial(mut remaining_probe) => {
                let mut new_interior = RadixInteriorNode {
                    children: NodeChildren::new(),
                    prefix: node.remaining_key,
                };

                node.remaining_key = KeyPrefix::empty();
                new_interior
                    .children
                    .insert_child(None, box RadixNode::Leaf(node));

                let next_char_new =
                    remaining_probe
                        .pop()
                        .expect(&format!("{}: {}", file!(), line!()));
                let new_leaf: RadixNode<K, V> = RadixNode::Leaf(RadixLeafNode {
                    remaining_key: From::from(remaining_probe),
                    entry: box new_entry,
                });

                debug_assert!(!new_interior.children.contains_child(next_char_new));
                new_interior
                    .children
                    .insert_child(Some(next_char_new), box new_leaf);

                (box RadixNode::Interior(new_interior), None)
            }
            KeyMatchResult::LongerPrefix(split_index) => {
                let (common, mut difference) = node.remaining_key.split_at(split_index);

                let mut new_interior = RadixInteriorNode {
                    prefix: common,
                    children: NodeChildren::new(),
                };

                let new_leaf = RadixNode::Leaf(RadixLeafNode {
                    remaining_key: KeyPrefix::empty(),
                    entry: box new_entry,
                });

                new_interior.children.insert_child(None, box new_leaf);

                let next_char = difference
                    .pop()
                    .expect(&format!("{}: {}", file!(), line!()));
                node.remaining_key = difference;
                new_interior
                    .children
                    .insert_child(Some(next_char), box RadixNode::Leaf(node));

                (box RadixNode::Interior(new_interior), None)
            }
            KeyMatchResult::Incomplete(split_index, mut remaining_probe) => {
                let (common, mut difference) = node.remaining_key.split_at(split_index);

                let mut new_interior = RadixInteriorNode {
                    prefix: common,
                    children: NodeChildren::new(),
                };

                let next_char_old = difference
                    .pop()
                    .expect(&format!("{}: {}", file!(), line!()));
                let next_char_new =
                    remaining_probe
                        .pop()
                        .expect(&format!("{}: {}", file!(), line!()));

                node.remaining_key = difference;
                let new_leaf = RadixLeafNode {
                    remaining_key: From::from(remaining_probe),
                    entry: box new_entry,
                };

                new_interior
                    .children
                    .insert_child(Some(next_char_old), box RadixNode::Leaf(node));
                new_interior
                    .children
                    .insert_child(Some(next_char_new), box RadixNode::Leaf(new_leaf));

                (box RadixNode::Interior(new_interior), None)
            }
        },
        RadixNode::Interior(mut node) => match node.prefix.match_with(probe) {
            KeyMatchResult::Complete => {
                if node.children.contains_empty() {
                    let old_node = node.children.remove_child(None).expect(&format!(
                        "{}: {}",
                        file!(),
                        line!()
                    ));

                    let (updated, replaced_value) =
                        recursive_insert(old_node, KeyProbe::empty(), new_entry);

                    node.children.insert_child(None, updated);

                    (box RadixNode::Interior(node), replaced_value)
                } else {
                    let new_leaf = RadixNode::Leaf(RadixLeafNode {
                        remaining_key: KeyPrefix::empty(),
                        entry: box new_entry,
                    });

                    node.children.insert_child(None, box new_leaf);

                    (box RadixNode::Interior(node), None)
                }
            }
            KeyMatchResult::Partial(mut remaining_probe) => {
                let next_char = remaining_probe
                    .pop()
                    .expect(&format!("{}: {}", file!(), line!()));

                if node.children.contains_child(next_char) {
                    let old_node = node.children.remove_child(Some(next_char)).expect(&format!(
                        "{}: {}",
                        file!(),
                        line!()
                    ));

                    let (updated, replaced_value) =
                        recursive_insert(old_node, remaining_probe, new_entry);

                    node.children.insert_child(Some(next_char), updated);

                    (box RadixNode::Interior(node), replaced_value)
                } else {
                    let new_leaf = RadixNode::Leaf(RadixLeafNode {
                        remaining_key: From::from(remaining_probe),
                        entry: box new_entry,
                    });

                    node.children.insert_child(Some(next_char), box new_leaf);

                    (box RadixNode::Interior(node), None)
                }
            }
            KeyMatchResult::LongerPrefix(split_index) => {
                let (common, mut difference) = node.prefix.split_at(split_index);

                let mut new_interior = RadixInteriorNode {
                    prefix: common,
                    children: NodeChildren::new(),
                };

                let new_leaf = RadixNode::Leaf(RadixLeafNode {
                    entry: box new_entry,
                    remaining_key: KeyPrefix::empty(),
                });

                new_interior.children.insert_child(None, box new_leaf);

                let next_char = difference
                    .pop()
                    .expect(&format!("{}: {}", file!(), line!()));
                node.prefix = difference;

                new_interior
                    .children
                    .insert_child(Some(next_char), box RadixNode::Interior(node));

                (box RadixNode::Interior(new_interior), None)
            }
            KeyMatchResult::Incomplete(split_index, mut remaining_probe) => {
                let (common, mut difference) = node.prefix.split_at(split_index);

                let next_char_old = difference
                    .pop()
                    .expect(&format!("{}: {}", file!(), line!()));
                node.prefix = difference;
                let next_char_new =
                    remaining_probe
                        .pop()
                        .expect(&format!("{}: {}", file!(), line!()));

                let mut new_interior = RadixInteriorNode {
                    prefix: common,
                    children: NodeChildren::new(),
                };

                let new_leaf = RadixLeafNode {
                    remaining_key: From::from(remaining_probe),
                    entry: box new_entry,
                };

                new_interior
                    .children
                    .insert_child(Some(next_char_old), box RadixNode::Interior(node));
                new_interior
                    .children
                    .insert_child(Some(next_char_new), box RadixNode::Leaf(new_leaf));

                (box RadixNode::Interior(new_interior), None)
            }
        },
    }
}

pub fn recursive_find<'p, 'v, K: TreeKey, V>(
    current: &'v Box<RadixNode<K, V>>,
    probe: KeyProbe<'p>,
) -> Option<&'v V> {
    match **current {
        RadixNode::Interior(ref node) => match node.prefix.match_with(probe) {
            KeyMatchResult::Complete => {
                if node.children.contains_empty() {
                    let child =
                        node.children
                            .get_child(None)
                            .expect(&format!("{}: {}", file!(), line!()));
                    debug_assert!(child.is_leaf());
                    debug_assert!(child.get_leaf().remaining_key.is_empty());

                    Some(child.get_leaf().entry.value())
                } else {
                    None
                }
            }
            KeyMatchResult::Partial(mut remaining_probe) => {
                let next_char = remaining_probe
                    .pop()
                    .expect(&format!("{}: {}", file!(), line!()));
                if node.children.contains_child(next_char) {
                    return recursive_find(
                        node.children.get_child(Some(next_char)).expect(&format!(
                            "{}: {}",
                            file!(),
                            line!()
                        )),
                        remaining_probe,
                    );
                } else {
                    None
                }
            }
            _ => None,
        },
        RadixNode::Leaf(ref node) => match node.remaining_key.match_with(probe) {
            KeyMatchResult::Complete => Some(node.entry.value()),
            _ => None,
        },
    }
}

pub fn recursive_mut_find<'p, 'v, K: TreeKey, V>(
    current: &'v mut Box<RadixNode<K, V>>,
    probe: KeyProbe<'p>,
) -> Option<&'v mut V> {
    match **current {
        RadixNode::Interior(ref mut node) => match node.prefix.match_with(probe) {
            KeyMatchResult::Complete => {
                if node.children.contains_empty() {
                    let child = node.children.get_child_mut(None).expect(&format!(
                        "{}: {}",
                        file!(),
                        line!()
                    ));
                    debug_assert!(child.is_leaf());
                    debug_assert!(child.get_leaf().remaining_key.is_empty());

                    Some(child.get_leaf_mut().entry.value_mut())
                } else {
                    None
                }
            }
            KeyMatchResult::Partial(mut remaining_probe) => {
                let next_char = remaining_probe
                    .pop()
                    .expect(&format!("{}: {}", file!(), line!()));
                if node.children.contains_child(next_char) {
                    return recursive_mut_find(
                        node.children
                            .get_child_mut(Some(next_char))
                            .expect(&format!("{}: {}", file!(), line!())),
                        remaining_probe,
                    );
                } else {
                    None
                }
            }
            _ => None,
        },
        RadixNode::Leaf(ref mut node) => match node.remaining_key.match_with(probe) {
            KeyMatchResult::Complete => Some(node.entry.value_mut()),
            _ => None,
        },
    }
}

pub fn recursive_remove<'p, 'v, K: TreeKey, V>(
    current: Box<RadixNode<K, V>>,
    probe: KeyProbe<'p>,
) -> (Option<Box<RadixNode<K, V>>>, Option<V>) {
    match *current {
        RadixNode::Leaf(node) => match node.remaining_key.match_with(probe) {
            KeyMatchResult::Complete => (None, Some(node.entry.take_value())),
            _ => (Some(box RadixNode::Leaf(node)), None),
        },
        RadixNode::Interior(mut node) => match node.prefix.match_with(probe) {
            KeyMatchResult::Complete => {
                let removed_value = if node.children.contains_empty() {
                    let empty_child = node.children.remove_child(None).unwrap();

                    let (updated_empty, removed_value) =
                        recursive_remove(empty_child, KeyProbe::empty());

                    if let Some(updated_empty) = updated_empty {
                        node.children.insert_child(None, updated_empty);
                    }

                    removed_value
                } else {
                    None
                };

                (Some(box RadixNode::Interior(node)), removed_value)
            },
            KeyMatchResult::Partial(mut remaining_probe) => {
                let next_char = remaining_probe.pop().unwrap();

                let removed_value = if node.children.contains_child(next_char) {
                    let child = node.children.remove_child(Some(next_char)).unwrap();

                    let (updated_child, removed_value) = recursive_remove(child, remaining_probe);

                    if let Some(updated_child) = updated_child {
                        node.children.insert_child(Some(next_char), updated_child);
                    }

                    removed_value
                } else {
                    None
                };

                (Some(box RadixNode::Interior(node)), removed_value)
            }
            _ => (Some(box RadixNode::Interior(node)), None),
        },
    }
}

#[cfg(test)]
mod radix_node_tests {
    use super::*;

    #[test]
    fn new_leaf() {
        let node = RadixNode::new_leaf("hello", 10);

        assert_eq!(
            node,
            RadixNode::Leaf(RadixLeafNode {
                entry: box KeyValue::new("hello", 10),
                remaining_key: KeyPrefix::new(b"hello"),
            })
        );
    }
}

#[cfg(any(debug_assertions, test))]
pub mod debug {
    use std::fmt;
    use std::cell::Cell;
    use std::str;
    use std::iter;

    use super::RadixNode;
    use super::super::key::TreeKey;

    pub struct TreeView<'a, K, V>
    where
        K: 'a + TreeKey,
        V: 'a + fmt::Debug,
    {
        root: &'a Box<RadixNode<K, V>>,
        context: TreeViewContext,
    }

    impl<'a, K, V> TreeView<'a, K, V>
    where
        K: 'a + TreeKey,
        V: 'a + fmt::Debug,
    {
        pub fn new(root: &'a Box<RadixNode<K, V>>, indent_size: usize) -> Self {
            TreeView {
                root,
                context: TreeViewContext::new(indent_size),
            }
        }
    }

    impl<'a, K, V> fmt::Debug for TreeView<'a, K, V>
    where
        K: 'a + TreeKey + fmt::Debug,
        V: 'a + fmt::Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            recursive_tree_format(self.root, f, &self.context)
        }
    }

    pub struct TreeViewContext {
        indent: Cell<usize>,
        indent_size: usize,
    }

    impl TreeViewContext {
        fn new(indent_size: usize) -> Self {
            TreeViewContext {
                indent: Cell::new(0),
                indent_size,
            }
        }
    }

    impl Default for TreeViewContext {
        fn default() -> Self {
            TreeViewContext {
                indent: Cell::default(),
                indent_size: 7,
            }
        }
    }

    fn recursive_tree_format<'p, 'v, K: TreeKey, V>(
        current: &'v Box<RadixNode<K, V>>,
        f: &mut fmt::Formatter,
        context: &TreeViewContext,
    ) -> fmt::Result
    where
        K: fmt::Debug,
        V: fmt::Debug,
    {
        let indent: String = iter::repeat(" ")
            .take(context.indent.get() * context.indent_size)
            .collect();

        match **current {
            RadixNode::Interior(ref node) => {
                write!(
                    f,
                    "[{}]\n",
                    str::from_utf8(node.prefix.bytes()).expect(&format!(
                        "{}: {}",
                        file!(),
                        line!()
                    ))
                )?;

                context.indent.set(context.indent.get() + 1);

                if node.children.contains_empty() {
                    write!(f, "{}(-) -> ", indent)?;
                    let empty_child =
                        node.children
                            .get_child(None)
                            .expect(&format!("{}: {}", file!(), line!()));
                    recursive_tree_format(empty_child, f, &context)?;
                }

                for &(ref branch_char, ref child) in node.children.iter() {
                    write!(f, "{}({}) -> ", indent, *branch_char as char)?;
                    recursive_tree_format(child, f, &context)?;
                }

                context.indent.set(context.indent.get() - 1);

                Ok(())
            }
            RadixNode::Leaf(ref node) => write!(
                f,
                "{}: {:?}\n",
                str::from_utf8(node.remaining_key.bytes()).expect(&format!(
                    "{}: {}",
                    file!(),
                    line!()
                )),
                node.entry.value()
            ),
        }
    }
}
