extern crate streamers;
extern crate unicode_segmentation;

use std::io::{self, Read};
use std::fs;
use std::env;

use unicode_segmentation::UnicodeSegmentation;

use streamers::radix_tree::RadixTree;

#[test]
fn basic_insert_retrieve() {
    let mut rax: RadixTree<&str, usize> = RadixTree::new();

    rax.insert("he", 10);
    rax.insert("ha", 11);
    rax.insert("hi", 12);
    rax.insert("hell", 20);
    rax.insert("hill", 21);
    rax.insert("hall", 22);
    rax.insert("hella", 30);
    rax.insert("hello", 31);

    assert_eq!(rax.get(&"he"), Some(&10));
    assert_eq!(rax.get(&"ha"), Some(&11));
    assert_eq!(rax.get(&"hi"), Some(&12));
    assert_eq!(rax.get(&"hell"), Some(&20));
    assert_eq!(rax.get(&"hill"), Some(&21));
    assert_eq!(rax.get(&"hall"), Some(&22));
    assert_eq!(rax.get(&"hella"), Some(&30));
    assert_eq!(rax.get(&"hello"), Some(&31));

    println!("{:?}", rax.debug_view());
}

fn read_file_into_words(filename: &str) -> io::Result<Vec<String>> {
    let mut file = fs::File::open(filename)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let words = contents
        .unicode_words()
        .map(From::from)
        .collect::<Vec<String>>();

    Ok(words)
}

#[test]
#[ignore]
fn insert_large_file_words_inferno() {
    let words = read_file_into_words("./assets/inferno.txt")
        .expect(&format!("Load file failed. Cwd {:?}", env::current_dir()));
    let mut rax = RadixTree::new();

    for word in words.iter() {
        rax.insert(word, ());
    }

    assert!(rax.len() < words.iter().count());
}

#[test]
#[ignore]
fn insert_large_file_words_don_quixote() {
    let words = read_file_into_words("./assets/don_quixote.txt")
        .expect(&format!("Load file failed. Cwd {:?}", env::current_dir()));
    let mut rax = RadixTree::new();

    for word in words.iter() {
        rax.insert(word, ());
    }

    assert!(rax.len() < words.iter().count());
}

use std::collections::{HashMap, HashSet};

#[test]
#[ignore]
fn insert_large_file_words_word_list() {
    let words = read_file_into_words("./assets/words.txt")
        .expect(&format!("Load file failed. Cwd {:?}", env::current_dir()));
    let mut rax = RadixTree::new();

    let mut word_map = HashMap::new();
    for word in words.iter() {
        *word_map.entry(word).or_insert(0) += 1;
    }

    let mut word_set = HashSet::new();
    word_set.extend(words.iter());

    println!("{}", word_set.len());

    for (word, count) in word_map.iter().filter(|&(_, count)| *count > 1) {
        println!("{}: {}", word, count);
    }

    for word in words.iter() {
        rax.insert(word, ());
    }

    assert_eq!(rax.len(), words.iter().count());
}
