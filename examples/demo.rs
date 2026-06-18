use fuzzy_dict::{Alphabet, FuzzyDict};
use std::env;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <dictionary_file> <query> [limit]", args[0]);
        eprintln!("Example: {} words.txt cafe 15", args[0]);
        process::exit(1);
    }

    let dict_path = &args[1];
    let query = &args[2];
    let threshold = 0.7;
    let limit = args.get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(15);

    // --- Load the dictionary ---
    let start_load = Instant::now();
    let alphabet = Alphabet::default();
    let mut dict = FuzzyDict::with_alphabet(alphabet);

    if let Err(e) = dict.add_file(dict_path) {
        eprintln!("Error reading dictionary file '{}': {}", dict_path, e);
        process::exit(1);
    }
    let load_duration = start_load.elapsed();

    let total_words = dict.word_count();
    let per_word_us = if total_words > 0 {
        load_duration.as_secs_f64() * 1_000_000.0 / total_words as f64
    } else {
        0.0
    };

    // --- Perform search with limit ---
    let start_query = Instant::now();
    let results = dict.search_limit(query, threshold, limit);
    let query_duration = start_query.elapsed();

    // --- Output results ---
    println!("Dictionary loaded from '{}'", dict_path);
    println!("  Total words: {}", total_words);
    println!("  Load time:   {:.2} ms (≈ {:.2} µs per word)",
             load_duration.as_secs_f64() * 1000.0,
             per_word_us);
    println!("\nQuery: '{}' (threshold = {:.2}, limit = {})", query, threshold, limit);
    println!("  Search time: {:.3} ms", query_duration.as_secs_f64() * 1000.0);

    if results.is_empty() {
        println!("  No matches found.");
    } else {
        println!("  Found {} matches:", results.len());
        for (word, score) in results {
            println!("    {} -> {:.3}", word, score);
        }
    }
}