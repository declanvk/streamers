use std::borrow::Borrow;
use std::cmp;
use std::fmt;
use std::str;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPrefix {
    prefix: Box<[u8]>
}

impl KeyPrefix {
    pub fn new(key_bytes: &[u8]) -> KeyPrefix {
        KeyPrefix {
            prefix: Box::from(key_bytes),
        }
    }

    pub fn empty() -> KeyPrefix {
        KeyPrefix {
            prefix: Box::new([]),
        }
    }

    pub fn len(&self) -> usize {
        self.prefix.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prefix.len() == 0
    }

    pub fn bytes(&self) -> &[u8] {
        self.prefix.borrow()
    }

    // This operation will copy the data
    // FUTURE WORK: implement a method that will split without
    // needing to copy
    pub fn split_at(self, idx: usize) -> (KeyPrefix, KeyPrefix) {
        let borrowed: &[u8] = self.prefix.borrow();
        let (left, right) = borrowed.split_at(idx);
        (KeyPrefix::new(left), KeyPrefix::new(right))
    }

    // Also inefficient
    pub fn pop(&mut self) -> Option<u8> {
        if !self.prefix.is_empty() {
            let mut prefix_vec = self.prefix.to_vec();
            let first_value = prefix_vec.pop().unwrap();

            self.prefix = prefix_vec.into_boxed_slice();
            Some(first_value)
        } else {
            None
        }
    }

    pub fn match_with<'a>(&self, probe: KeyProbe<'a>) -> KeyMatchResult<'a> {
        let byte_prefix: &[u8] = self.prefix.borrow();
        let is_prefix = probe.bytes().starts_with(byte_prefix);

        if is_prefix {
            if probe.len() > self.len() {
                let (_, right) = probe.split_at(self.len());
                KeyMatchResult::Partial(right)
            } else {
                debug_assert_eq!(probe.len(), self.len());
                KeyMatchResult::Complete
            }
        } else if probe.len() < self.len() && byte_prefix.starts_with(probe.bytes()) {
            KeyMatchResult::LongerPrefix(probe.len())
        } else {
            let diff_index = self.diff_index(&probe).unwrap();

            if diff_index == 0 {
                KeyMatchResult::Incomplete(0, probe)
            } else {
                let (_, right) = probe.split_at(diff_index);
                KeyMatchResult::Incomplete(diff_index, right)
            }
        }
    }

    fn diff_index<'a>(&self, probe: &KeyProbe<'a>) -> Option<usize> {
        let max_len = cmp::max(self.len(), probe.len());
        let prefix_bytes: &[u8] = self.prefix.borrow();
        let probe_bytes: &[u8] = probe.bytes();

        for idx in 0..max_len {
            if idx >= self.len() || idx >= probe.len() || prefix_bytes[idx] != probe_bytes[idx] {
                return Some(idx);
            }
        }

        None
    }
}

impl<'a> From<KeyProbe<'a>> for KeyPrefix {
    fn from(src: KeyProbe<'a>) -> Self {
        KeyPrefix::new(src.bytes())
    }
}

impl fmt::Display for KeyPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match str::from_utf8(self.prefix.borrow()) {
            Ok(val) => write!(f, "{}", val),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyMatchResult<'a> {
    // Occurs when the prefix matches the start of the probe completely,
    // and the prefix is also shorter than the probe
    // Returns the end of the probe that did not match
    // Example
    // Prefix: "ABC"
    // Probe:  "ABCDEF"
    // The result should be Partial("DEF")
    Partial(KeyProbe<'a>),

    // Occurs when the prefix and the probe are the same length and are equal
    // Returns nothing as the probe has been completely consumed
    // Example
    // Prefix: "ABC"
    // Probe:  "ABC"
    // The result should be Complete
    Complete,

    // Occurs when the prefix is longer, but the probe is completely matched
    // against a prefix of the prefix.
    // Returns the index at which to split the prefix
    // Example
    // Prefix: "ABCDEF"
    // Probe:  "ABC"
    // The result should be LongerPrefix(3)
    LongerPrefix(usize),

    // This is the catchall for all nonmatching prefixes. This will occur when
    // a single character does not match the probe.
    // Returns the index to split the prefix at and the remnants of the probe

    // Example 1 - match diverges in the middle
    // Prefix: "ABZDEF"
    // Probe:  "ABCDEF"
    // The result should be Incomplete(2, "CDEF")

    // Example 2 - no portion matches at all
    // Prefix: "ABCDEF"
    // Probe:  "GHIJKL"
    // The result should be Incomplete(0, "GHIJKL")

    // Example 2 - the probe is too short and a portion doesn't match
    // Prefix: "ABCDEF"
    // Probe:  "ABZ"
    // The result should be Incomplete(2, "Z")
    Incomplete(usize, KeyProbe<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyProbe<'a> {
    key_portion: &'a [u8],
}

impl<'a> KeyProbe<'a> {
    pub fn new<K>(key: &'a K) -> Self
    where
        K: TreeKey,
    {
        KeyProbe {
            key_portion: key.as_bytes(),
        }
    }

    pub fn empty() -> Self {
        KeyProbe { key_portion: &[] }
    }

    pub fn len(&self) -> usize {
        self.key_portion.len()
    }

    pub fn is_empty(&self) -> bool {
        self.key_portion.len() == 0
    }

    pub fn bytes(&self) -> &[u8] {
        self.key_portion
    }

    pub fn split_at(self, idx: usize) -> (KeyProbe<'a>, KeyProbe<'a>) {
        let (left_bytes, right_bytes) = self.key_portion.split_at(idx);
        let left = KeyProbe {
            key_portion: left_bytes,
        };
        let right = KeyProbe {
            key_portion: right_bytes,
        };

        (left, right)
    }

    pub fn pop(&mut self) -> Option<u8> {
        if let Some((first, rest)) = self.key_portion.split_first() {
            self.key_portion = rest;
            Some(*first)
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for KeyProbe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match str::from_utf8(self.key_portion) {
            Ok(val) => write!(f, "{}", val),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[cfg(test)]
mod key_prefix_tests {
    use super::*;

    #[test]
    fn new_prefix() {
        let prefix = KeyPrefix::new(b"ABC");

        assert_eq!(prefix.len(), 3);
        assert_eq!(prefix.bytes(), b"ABC");
    }

    #[test]
    fn split_prefix() {
        let prefix = KeyPrefix::new(b"ABCDEFG");
        assert_eq!(prefix.len(), 7);
        assert_eq!(prefix.bytes(), b"ABCDEFG");

        let (left, right) = prefix.split_at(3);
        assert_eq!(left.len(), 3);
        assert_eq!(left.bytes(), b"ABC");

        assert_eq!(right.len(), 4);
        assert_eq!(right.bytes(), b"DEFG");
    }

    #[test]
    fn prefix_match_empty() {
        let prefix_a = KeyPrefix::new(b"");
        let probe_a = KeyProbe::new(&"ABC");

        let match_result = prefix_a.match_with(probe_a);
        assert_eq!(match_result, KeyMatchResult::Partial(KeyProbe::new(&"ABC")));
    }

    #[test]
    fn prefix_match_partial() {
        let prefix_a = KeyPrefix::new(b"ABC");
        let probe_a = KeyProbe::new(&"ABCDEF");

        let match_result = prefix_a.match_with(probe_a);
        assert_eq!(match_result, KeyMatchResult::Partial(KeyProbe::new(&"DEF")));
    }

    #[test]
    fn prefix_match_full() {
        let prefix_a = KeyPrefix::new(b"ABCDEF");
        let probe_a = KeyProbe::new(&"ABCDEF");

        let match_result = prefix_a.match_with(probe_a);
        assert_eq!(match_result, KeyMatchResult::Complete);
    }

    #[test]
    fn prefix_match_longer_prefix() {
        let prefix_a = KeyPrefix::new(b"ABCDEFGHI");
        let probe_a = KeyProbe::new(&"ABCDEF");

        let match_result = prefix_a.match_with(probe_a);
        assert_eq!(match_result, KeyMatchResult::LongerPrefix(6));
    }

    #[test]
    fn prefix_match_incomplete() {
        let prefix_a = KeyPrefix::new(b"ABZDEF");
        let probe_a = KeyProbe::new(&"ABCDEF");

        let match_result = prefix_a.match_with(probe_a);
        assert_eq!(
            match_result,
            KeyMatchResult::Incomplete(2, KeyProbe::new(&"CDEF"))
        );

        let prefix_b = KeyPrefix::new(b"ABCDEF");
        let probe_b = KeyProbe::new(&"GHIJKL");

        let match_result = prefix_b.match_with(probe_b);
        assert_eq!(
            match_result,
            KeyMatchResult::Incomplete(0, KeyProbe::new(&"GHIJKL"))
        );

        let prefix_c = KeyPrefix::new(b"ABCDEF");
        let probe_c = KeyProbe::new(&"ABZ");

        let match_result = prefix_c.match_with(probe_c);
        assert_eq!(
            match_result,
            KeyMatchResult::Incomplete(2, KeyProbe::new(&"Z"))
        );
    }
}

#[cfg(test)]
mod key_probe_tests {
    use super::*;

    #[test]
    fn new_probe() {
        let probe = KeyProbe::new(&"ABC");
        assert_eq!(probe.len(), 3);
        assert_eq!(probe.bytes(), b"ABC");
    }

    #[test]
    fn empty_probe() {
        let probe = KeyProbe::new(&"");

        assert!(probe.is_empty());
        assert_eq!(probe.len(), 0);
        assert_eq!(probe.bytes(), b"");
    }

    #[test]
    fn probe_split() {
        let probe = KeyProbe::new(&"ABCDEFG");
        assert_eq!(probe.len(), 7);
        assert_eq!(probe.bytes(), b"ABCDEFG");

        let (left, right) = probe.split_at(3);
        assert_eq!(left.len(), 3);
        assert_eq!(left.bytes(), b"ABC");

        assert_eq!(right.len(), 4);
        assert_eq!(right.bytes(), b"DEFG");
    }

    #[test]
    fn probe_split_extrema() {
        let probe_a = KeyProbe::new(&"ABCDEFG");
        let (left_a, right_a) = probe_a.split_at(0);
        assert!(left_a.is_empty());

        assert_eq!(right_a.len(), 7);
        assert_eq!(right_a.bytes(), b"ABCDEFG");

        let probe_b = KeyProbe::new(&"ABCDEFG");
        let (left_b, right_b) = probe_b.split_at(7);
        assert_eq!(left_b.len(), 7);
        assert_eq!(left_b.bytes(), b"ABCDEFG");

        assert!(right_b.is_empty());
    }

    #[test]
    fn probe_pop() {
        let mut probe = KeyProbe::new(&"ABCDEFG");
        let mut reversed = Vec::new();
        while probe.len() > 0 {
            reversed.insert(0, probe.pop().unwrap() as char);
        }

        assert_eq!(reversed, ['G', 'F', 'E', 'D', 'C', 'B', 'A']);
    }
}

pub trait TreeKey: Clone + PartialEq + Eq {
    fn as_bytes(&self) -> &[u8];
    // fn from_bytes(key_bytes: &[u8]) -> Self;
}

impl<T> TreeKey for T
where
    T: AsRef<[u8]> + Clone + Eq,
{
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}
