use fuzzy_dict::{Alphabet, FuzzyDict};
use std::env;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <dictionary_file> <query> [limit] [score_threshold]", args[0]);
        eprintln!("Example: {} words.txt cafe 15 0.7", args[0]);
        process::exit(1);
    }

    let dict_path = &args[1];
    let query = &args[2];
    let limit = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(15);
    let threshold = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0);

    // --- Load ---
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

    // --- Stats ---
    let bucket_count = dict.bucket_count();
    let largest_bucket = dict.largest_bucket_size();
    let singleton_ratio = dict.singleton_bucket_ratio() * 100.0;

    // --- Warm-up (optional) ---
    // let _ = dict.search_limit("", 1); // discard

    // --- Search ---
    let start_query = Instant::now();
    let mut results = dict.search_limit(query, limit);
    // Filter by threshold (if > 0)
    if threshold > 0.0 {
        results.retain(|(_, s)| *s >= threshold);
    }
    let query_duration = start_query.elapsed();

    // --- Output ---
    println!("Dictionary loaded from '{}'", dict_path);
    println!("  Total words:      {} words", total_words);
    println!("  Total buckets:    {} buckets", bucket_count);
    println!("  Largest bucket:   {} words", largest_bucket);
    println!("  Singleton buckets: {:.1}%", singleton_ratio);
    println!("  Load time:        {:.3} ms (≈ {:.3} µs per word)",
             load_duration.as_secs_f64() * 1000.0,
             per_word_us);
    println!("\nQuery: '{}' (limit = {}, score filter = {:.2})", query, limit, threshold);
    println!("  Search time: {:.3} ms", query_duration.as_secs_f64() * 1000.0);

    if results.is_empty() {
        println!("  No matches found (or none above threshold).");
    } else {
        println!("  Found {} matches:", results.len());
        for (word, score) in results {
            println!("    {} -> {:.3}", word, score);
        }
    }
}