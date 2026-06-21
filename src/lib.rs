mod alphabet;

pub use alphabet::{Alphabet, UNKNOWN_BIT};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A dictionary that uses character‑presence bitmasks for fast fuzzy searching.
/// Words are stored in buckets keyed by their mask. Search expands from exact
/// mask to 1‑bit and 2‑bit flips, collecting at least `MIN_RESERVE` candidates,
/// then scores them with a fast inline scorer and returns the top `limit` results.
pub struct FuzzyDict {
    alphabet: Alphabet,
    buckets: HashMap<u64, Vec<String>>,
    effective_mask: u64,
    total_words: usize,
}

impl FuzzyDict {
    /// Returns the number of non‑empty buckets.
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Returns the size of the largest bucket (number of words).
    pub fn largest_bucket_size(&self) -> usize {
        self.buckets.values().map(|v| v.len()).max().unwrap_or(0)
    }

    /// Returns the fraction of buckets that contain exactly one word.
    pub fn singleton_bucket_ratio(&self) -> f64 {
        let total = self.buckets.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let singletons = self.buckets.values().filter(|v| v.len() == 1).count() as f64;
        singletons / total
    }

    /// Creates a new empty `FuzzyDict` with the given alphabet.
    pub fn with_alphabet(alphabet: Alphabet) -> Self {
        FuzzyDict {
            alphabet,
            buckets: HashMap::new(),
            effective_mask: 0,
            total_words: 0,
        }
    }

    /// Adds a single word to the dictionary.
    pub fn add_word(&mut self, word: &str) {
        let mask = self.alphabet.word_mask(word);
        self.buckets.entry(mask).or_insert_with(Vec::new).push(word.to_string());
        self.effective_mask |= mask;
        self.total_words += 1;
    }

    /// Reads words from a file (one per line, empty lines and lines starting with '#' are skipped).
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

    /// Adds multiple words from an iterator.
    pub fn extend<I>(&mut self, words: I)
    where
        I: IntoIterator<Item = String>,
    {
        for word in words {
            self.add_word(&word);
        }
    }

    /// Returns the total number of words in the dictionary.
    pub fn word_count(&self) -> usize {
        self.total_words
    }

    /// Fast inline scorer: longest common prefix + suffix divided by max length, capped at 1.0.
    /// This is used internally by `search_limit`.
    fn simple_score(a: &str, b: &str) -> f64 {
        let max_len = std::cmp::max(a.len(), b.len()) as f64;
        let min_len = std::cmp::min(a.len(), b.len()) as f64;
        if max_len == 0.0 {
            return 1.0;
        }
        let lcp = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count() as f64;
        let lcs = a.chars().rev().zip(b.chars().rev()).take_while(|(x, y)| x == y).count() as f64;
        ((lcp + lcs) / max_len).min(min_len/max_len)
    }

    /// Search with a limit on the number of results returned.
    /// Returns at most `limit` matches, sorted by score descending.
    /// No threshold is applied – all candidates from the expanded buckets are scored.
    ///
    /// The search proceeds in order:
    /// 1. Exact mask bucket.
    /// 2. 1‑bit flip buckets.
    /// 3. 2‑bit flip buckets.
    ///
    /// It stops collecting as soon as at least `MIN_RESERVE` candidates are gathered.
    /// If any word scores exactly 1.0 in the exact bucket, it is returned immediately
    /// as the sole result (single perfect match).
    pub fn search_limit(&self, query: &str, limit: usize) -> Vec<(String, f64)> {
        const SCORE_SUM_THRESHOLD: f64 = 15.0;  // tunable

        let query_mask = self.alphabet.word_mask(query);

        // Build masks: exact, 1-flip, 2-flip
        let mut masks_to_check = Vec::new();
        masks_to_check.push(query_mask);

        let mut effective_bits: Vec<usize> = (0..64)
            .filter(|&i| (self.effective_mask >> i) & 1 == 1)
            .collect();
        if !effective_bits.contains(&UNKNOWN_BIT) {
            effective_bits.push(UNKNOWN_BIT);
        }

        for &bit in &effective_bits {
            masks_to_check.push(query_mask ^ (1 << bit));
        }
        let n = effective_bits.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let bit1 = 1 << effective_bits[i];
                let bit2 = 1 << effective_bits[j];
                masks_to_check.push(query_mask ^ bit1 ^ bit2);
            }
        }

        // Deduplicate (preserve order)
        let mut seen = HashSet::new();
        let masks: Vec<u64> = masks_to_check
            .into_iter()
            .filter(|&m| seen.insert(m))
            .collect();

        let mut results = Vec::new();
        let mut score_sum = 0.0;

        // Early perfect match detection: we'll check after collecting exact bucket,
        // but we can also check during collection for the exact mask.
        // We'll collect all, then check if any has 1.0 from the exact bucket.
        // To do that, we need to know which bucket each word came from? Not necessary:
        // we just check after collection if any score == 1.0 and if that word came from exact.
        // However, since exact is first, we can just collect all and then see if any has 1.0
        // and if we collected from exact. Simpler: we can just collect and then check if any
        // score == 1.0, and if the first element (or any) has that, we return it.
        // But we want to return only if it's from exact bucket. So we need to track bucket origin.
        // Alternatively, we can first examine exact bucket alone, and if any score == 1.0,
        // return it immediately. That's simpler and matches the requirement.

        // So we'll process exact bucket separately.
        if let Some(bucket) = self.buckets.get(&query_mask) {
            for word in bucket {
                let score = Self::simple_score(query, word);
                if score == 1.0 {
                    return vec![(word.clone(), 1.0)];
                }
                results.push((word.clone(), score));
                score_sum += score;
            }
            if score_sum >= SCORE_SUM_THRESHOLD {
                // We have enough quality; proceed to sort and return.
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                results.truncate(limit);
                return results;
            }
        }

        // Now process 1-flip and 2-flip masks (skip exact, already done)
        for &mask in &masks[1..] { // skip first (exact)
            if let Some(bucket) = self.buckets.get(&mask) {
                for word in bucket {
                    let score = Self::simple_score(query, word);
                    results.push((word.clone(), score));
                    score_sum += score;
                    if score_sum >= SCORE_SUM_THRESHOLD {
                        // Stop collecting further
                        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                        results.truncate(limit);
                        return results;
                    }
                }
            }
        }

        // If we exhaust all masks, sort and return what we have
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    /// Convenience method that returns all matches (no limit).
    pub fn search(&self, query: &str) -> Vec<(String, f64)> {
        self.search_limit(query, usize::MAX)
    }
}