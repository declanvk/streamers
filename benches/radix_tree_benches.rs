#[macro_use]
extern crate criterion;
extern crate unicode_segmentation;

extern crate streamers;

use criterion::Criterion;
use unicode_segmentation::UnicodeSegmentation;
use streamers::radix_tree::tree::RadixTree;

use std::io::{self, Read};
use std::fs;
use std::iter;

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

const BASE_NUM: usize = 100;
const NUM_GROUPS: usize = 5;

fn radix_build_tree(c: &mut Criterion) {
    let input_sizes = iter::repeat(BASE_NUM).take(NUM_GROUPS).enumerate().map(|(idx, value)| (idx + 1) * value).collect::<Vec<usize>>();
    c.bench_function_over_inputs("build radix tree from large file", |b, n| {
        let inferno_words = read_file_into_words("assets/inferno.txt").expect("Loading file failed");
        b.iter(|| {
            let mut rax = RadixTree::new();

            for word in inferno_words.iter().take(*n) {
                rax.insert(word, ());
            }
        })
    }, input_sizes);
}

criterion_group!(radix_tree_benches, radix_build_tree);
criterion_main!(radix_tree_benches);
