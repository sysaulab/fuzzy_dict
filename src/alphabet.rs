use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Bit reserved for any character not explicitly defined in the alphabet.
pub const UNKNOWN_BIT: usize = 0;

/// A LUT that maps characters to bit positions.
/// Bits are assigned by line number: line 0 → bit 1, line 1 → bit 2, … up to bit 63.
/// All sources (standard or user files) are merged line‑wise.
pub struct Alphabet {
    char_to_bit: HashMap<char, usize>,
}

// ─── Embedded alphabet definitions ───────────────────────────────────────────

const LATIN_ALPHABET: &str = include_str!("../assets/latin.txt");
const CYRILLIC_ALPHABET: &str = include_str!("../assets/cyrillic.txt");
const GREEK_ALPHABET: &str = include_str!("../assets/greek.txt");
const ARABIC_ALPHABET: &str = include_str!("../assets/arabic.txt");
const HEBREW_ALPHABET: &str = include_str!("../assets/hebrew.txt");
const ARMENIAN_ALPHABET: &str = include_str!("../assets/armenian.txt");
const GEORGIAN_ALPHABET: &str = include_str!("../assets/georgian.txt");
const THAI_ALPHABET: &str = include_str!("../assets/thai.txt");
const DEVANAGARI_ALPHABET: &str = include_str!("../assets/devanagari.txt");
const ADLAM_ALPHABET: &str = include_str!("../assets/adlam.txt");
const COPTIC_ALPHABET: &str = include_str!("../assets/coptic.txt");
const ETHIOPIC_ALPHABET: &str = include_str!("../assets/ethiopic.txt");
const NKO_ALPHABET: &str = include_str!("../assets/nko.txt");
const TIFINAGH_ALPHABET: &str = include_str!("../assets/tifinagh.txt");
const BAYBAYIN_ALPHABET: &str = include_str!("../assets/baybayin.txt");
const BURMESE_ALPHABET: &str = include_str!("../assets/burmese.txt");
const JAVANESE_ALPHABET: &str = include_str!("../assets/javanese.txt");
const KHMER_ALPHABET: &str = include_str!("../assets/khmer.txt");
const LAO_ALPHABET: &str = include_str!("../assets/lao.txt");
const BENGALI_ALPHABET: &str = include_str!("../assets/bengali.txt");
const GUJARATI_ALPHABET: &str = include_str!("../assets/gujarati.txt");
const GURMUKHI_ALPHABET: &str = include_str!("../assets/gurmukhi.txt");
const TAMIL_ALPHABET: &str = include_str!("../assets/tamil.txt");
const TELUGU_ALPHABET: &str = include_str!("../assets/telugu.txt");

const ALL_STANDARD: &[(&str, &str)] = &[
    //europe
    ("latin", LATIN_ALPHABET),
    ("greek", GREEK_ALPHABET),
    ("armenian", ARMENIAN_ALPHABET),
    //india
    ("devanagari", DEVANAGARI_ALPHABET),
    ("bengali", BENGALI_ALPHABET),
    ("gujarati", GUJARATI_ALPHABET),
    ("gurmukhi", GURMUKHI_ALPHABET),
    ("tamil", TAMIL_ALPHABET),
    ("telugu", TELUGU_ALPHABET),
    //east-asia
    ("arabic", ARABIC_ALPHABET),
    ("hebrew", HEBREW_ALPHABET),
    //slavic
    ("cyrillic", CYRILLIC_ALPHABET),
    ("georgian", GEORGIAN_ALPHABET),
    //africa
    ("tifinagh", TIFINAGH_ALPHABET),
    ("nko", NKO_ALPHABET),
    ("ethiopic", ETHIOPIC_ALPHABET),
    ("coptic", COPTIC_ALPHABET),
    ("adlam", ADLAM_ALPHABET),
    //se-asia
    ("thai", THAI_ALPHABET),
    ("baybayin", BAYBAYIN_ALPHABET),
    ("burmese", BURMESE_ALPHABET),
    ("javanese", JAVANESE_ALPHABET),
    ("khmer", KHMER_ALPHABET),
    ("lao", LAO_ALPHABET),
];

impl Alphabet {
    /// Creates an empty alphabet (only the unknown bit is defined).
    pub fn new() -> Self {
        Alphabet {
            char_to_bit: HashMap::new(),
        }
    }

    /// Loads all standard alphabets (Latin, Cyrillic, Greek) merged line‑wise.
    pub fn load_standard() -> Self {
        let contents: Vec<&str> = ALL_STANDARD.iter().map(|(_, content)| *content).collect();
        Alphabet::from_contents(contents)
    }

    /// Loads only the named standard alphabets.
    /// Names: "latin", "cyrillic", "greek". Unknown names are ignored.
    pub fn load_named(names: &[&str]) -> Self {
        let mut contents = Vec::new();
        for name in names {
            if let Some(content) = ALL_STANDARD.iter().find_map(|(n, c)| {
                if n == name { Some(*c) } else { None }
            }) {
                contents.push(content);
            }
        }
        Alphabet::from_contents(contents)
    }

    /// Loads a single user‑defined alphabet file.
    /// Each line in the file defines one bit (line 0 → bit 1).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Alphabet::from_contents(vec![&content]))
    }

    /// Loads multiple user‑defined alphabet files, merging them line‑wise.
    pub fn from_files<P: AsRef<Path>>(paths: &[P]) -> Result<Self, std::io::Error> {
        let mut contents = Vec::new();
        for path in paths {
            let content = std::fs::read_to_string(path)?;
            contents.push(content);
        }
        // Convert to &str slices
        let content_refs: Vec<&str> = contents.iter().map(|s| s.as_str()).collect();
        Ok(Alphabet::from_contents(content_refs))
    }

    /// Internal: builds the alphabet from a list of string contents, merging line‑wise.
    fn from_contents(contents: Vec<&str>) -> Self {
        let mut lines: Vec<HashSet<char>> = Vec::new();

        for content in contents {
            for (idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if idx >= lines.len() {
                    lines.resize(idx + 1, HashSet::new());
                }
                for ch in trimmed.chars() {
                    lines[idx].insert(ch);
                }
            }
        }

        // Assign bits: line 0 → bit 1, line 1 → bit 2, … up to bit 63.
        let mut char_to_bit = HashMap::new();
        let max_bits = lines.len().min(63); // we only have 63 explicit bits (1..63)
        if lines.len() > 63 {
            panic!("Too many alphabet lines: maximum 63 explicit bits (bit 0 is reserved)");
        }
        for (line_idx, chars) in lines.iter().enumerate().take(max_bits) {
            let bit = line_idx + 1; // because bit 0 is unknown
            for &ch in chars {
                char_to_bit.insert(ch, bit);
            }
        }

        Alphabet { char_to_bit }
    }

    /// Returns the bit position for a character, if present.
    pub fn char_bit(&self, c: char) -> Option<usize> {
        self.char_to_bit.get(&c).copied()
    }

    /// Computes the mask for a word.
    /// Explicitly mapped characters set their respective bits.
    /// Any unmapped character sets the UNKNOWN_BIT (0).
    pub fn word_mask(&self, word: &str) -> u64 {
        let mut mask = 0;
        let mut has_unknown = false;
        for c in word.chars() {
            if let Some(bit) = self.char_bit(c) {
                mask |= 1 << bit;
            } else {
                has_unknown = true;
            }
        }
        if has_unknown {
            mask |= 1 << UNKNOWN_BIT;
        }
        mask
    }

/*     /// Returns the number of explicitly defined bits (excluding the unknown bit).
    pub fn num_bits(&self) -> usize {
        // The maximum bit we assigned is the number of lines we processed.
        // We can compute by finding the max bit in char_to_bit, but it's simpler to track.
        // We'll compute it on the fly.
        let mut max_bit = 0;
        for &bit in self.char_to_bit.values() {
            if bit > max_bit {
                max_bit = bit;
            }
        }
        max_bit // because bits are contiguous from 1..max_bit
    }*/
}

impl Default for Alphabet {
    fn default() -> Self {
        Self::load_standard()
    }
}
