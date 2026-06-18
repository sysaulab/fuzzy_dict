mod alphabet;
mod score;

pub use alphabet::{Alphabet, UNKNOWN_BIT};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct FuzzyDict {
    alphabet: Alphabet,
    buckets: HashMap<u64, Vec<String>>,
    effective_mask: u64,
    total_words: usize,
}

impl FuzzyDict {
    pub fn with_alphabet(alphabet: Alphabet) -> Self {
        FuzzyDict {
            alphabet,
            buckets: HashMap::new(),
            effective_mask: 0,
            total_words: 0,
        }
    }

    pub fn add_word(&mut self, word: &str) {
        let mask = self.alphabet.word_mask(word);
        self.buckets.entry(mask).or_insert_with(Vec::new).push(word.to_string());
        self.effective_mask |= mask;
        self.total_words += 1;
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            self.add_word(trimmed);
        }
        Ok(())
    }

    pub fn extend<I>(&mut self, words: I)
    where
        I: IntoIterator<Item = String>,
    {
        for word in words {
            self.add_word(&word);
        }
    }

    pub fn word_count(&self) -> usize {
        self.total_words
    }

    /// Search with no limit – returns all matches.
    pub fn search(&self, query: &str, threshold: f64) -> Vec<(String, f64)> {
        self.search_limit(query, threshold, usize::MAX)
    }

    /// Search with a maximum number of results.
    /// Returns at most `limit` matches, sorted by score descending.
    pub fn search_limit(&self, query: &str, threshold: f64, limit: usize) -> Vec<(String, f64)> {
        let query_mask = self.alphabet.word_mask(query);
        let query_len = query.len();

        // 1. Gather all candidate masks (exact + 1‑bit + 2‑bit flips)
        let mut masks_to_check = HashSet::new();
        masks_to_check.insert(query_mask);

        let mut effective_bits: Vec<usize> = (0..64)
            .filter(|&i| (self.effective_mask >> i) & 1 == 1)
            .collect();
        if !effective_bits.contains(&UNKNOWN_BIT) {
            effective_bits.push(UNKNOWN_BIT);
        }

        for &bit in &effective_bits {
            masks_to_check.insert(query_mask ^ (1 << bit));
        }

        let n = effective_bits.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let bit1 = 1 << effective_bits[i];
                let bit2 = 1 << effective_bits[j];
                masks_to_check.insert(query_mask ^ bit1 ^ bit2);
            }
        }

        // 2. Collect candidate words (as &str) from the buckets
        let mut candidates: Vec<&str> = Vec::new();
        for &mask in &masks_to_check {
            if let Some(bucket) = self.buckets.get(&mask) {
                for word in bucket {
                    candidates.push(word.as_str());
                }
            }
        }

        // 3. Sort by length proximity to the query (heuristic to score likely matches first)
        candidates.sort_by_key(|w| (w.len() as isize - query_len as isize).abs());

        // 4. Score candidates and keep only the top `limit` matches
        let mut results = Vec::new();
        for word in candidates {
            if word == query {
                results.push((word.to_string(), 1.0));
            } else {
                let s = score::similarity(query, word);
                if s >= threshold {
                    results.push((word.to_string(), s));
                }
            }
            // If we already have enough results and the remaining candidates have
            // larger length difference, we could break early, but we keep it simple.
        }

        // 5. Sort by score descending and truncate to limit
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }
}