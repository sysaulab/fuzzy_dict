```markdown
# fuzzy_dict

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

**fuzzy_dict** is a high‑performance fuzzy string matching library for Rust. It uses a character‑presence bitmask filter to quickly narrow down the search space before applying a similarity scorer. The approach is **conservative** (no false negatives) and can reduce the number of candidates that need to be scored by 80‑95%, making it ideal for large dictionaries.

> This library is a weekend project, but it’s open source and ready to use. If you find it useful, feel free to polish and upload it to crates.io!

## Features

- **Blazing fast** – O(1) bucket lookup for exact masks, with optional expansion to 1‑ and 2‑bit flips.
- **Multilingual** – Supports Latin, Cyrillic, Greek, Arabic, Hebrew, Armenian, Georgian, Thai, and Devanagari scripts out of the box. Custom alphabets can be defined via simple text files.
- **Accent‑ and case‑insensitive** – Character classes group accented variants and both cases together.
- **Conservative filter** – Never misses a potential match (no false negatives).
- **Easy to use** – Add words from files or iterators, then search with a limit and optional threshold.
- **Lightweight** – Only ~8 bytes per dictionary entry overhead.

## How It Works

1. **Alphabet definition**: Each character is assigned to a bit position (1–63) based on the line it appears on in the alphabet files. Bit 0 is reserved for unknown characters.
2. **Mask computation**: For every word, a 64‑bit mask is computed where each bit indicates the presence of a character class.
3. **Bucketing**: Words are stored in a `HashMap<u64, Vec<String>>` keyed by their mask.
4. **Search**: Given a query:
   - Compute its mask.
   - Look up the exact mask bucket.
   - If not enough good candidates are found, also inspect buckets whose masks differ by exactly 1 or 2 bit flips (only for bits that exist in the dictionary).
   - Score candidates using a fast inline scorer (longest common prefix + suffix normalised by max length).
   - Return the top `limit` results sorted by score.

The search stops early once the sum of scores of collected candidates exceeds `SCORE_SUM_THRESHOLD` (default 15.0), ensuring we only score the most promising candidates.

## Usage

Add this to your `Cargo.toml` (until it's on crates.io, use the git repository):

```toml
[dependencies]
fuzzy_dict = { git = "https://github.com/yourusername/fuzzy_dict" }
```

Then in your code:

```rust
use fuzzy_dict::{Alphabet, FuzzyDict};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load default alphabet (Latin + Cyrillic + Greek + Arabic + …)
    let alphabet = Alphabet::default();
    let mut dict = FuzzyDict::with_alphabet(alphabet);

    // Add words from a file (one word per line, '#' for comments)
    dict.add_file("dictionary.txt")?;

    // Or add words manually
    dict.add_word("hello");
    dict.add_word("world");

    // Search with a limit of 10 and optional score threshold
    let results = dict.search_limit("helo", 10);
    // Or search without limit
    // let all = dict.search("helo");

    for (word, score) in results {
        println!("{} -> {:.3}", word, score);
    }

    Ok(())
}
```

### Command‑Line Demo

The repository includes a `demo.rs` that shows how to load a dictionary and query it from the command line:

```bash
cargo run --example demo words.txt cafe 15 0.7
```

Arguments: `<dictionary_file> <query> [limit] [score_threshold]`

The demo prints loading statistics, search time, and the results.

## Alphabet Files

Alphabet files are plain text files where each line defines a character class. For example:

```
aAáÀâÄãÃ
bB
cCçÇ
```

All characters on the same line share the same bit. The library includes several predefined alphabets in the `assets/` folder. You can also create your own and load them using `Alphabet::from_file()` or `Alphabet::from_files()`.

To load only specific standard alphabets:

```rust
let alphabet = Alphabet::load_named(&["latin", "cyrillic"]);
```

## Performance

On a dictionary of ~500,000 words, the filter achieves:

- **100,000+ queries per second** on modest hardware.
- **Candidate reduction of 80‑95%** before scoring.
- **Memory overhead**: ~8 bytes per word for the mask, plus bucket storage.

The search algorithm is constant time for the exact mask, and the expansion to 1‑ and 2‑bit flips is bounded by the number of effective bits (≤63), making it scale well even for large dictionaries.

## Customisation

You can tweak the internal score threshold by modifying the constant `SCORE_SUM_THRESHOLD` in the source (currently 15.0). This controls how many candidates are collected before sorting. A higher value may yield more accurate results at the cost of slightly more scoring.

The scorer itself is a simple inline function; you can replace it with a more sophisticated metric like Jaro‑Winkler if needed, but that will increase query time.

## Limitations

- **Order‑insensitive**: The filter ignores character order, so "abc" and "cba" produce the same mask. This is a trade‑off for speed; for order‑sensitive matching consider using n‑gram masks.
- **CJK and logographic scripts**: With thousands of characters, the 64‑bit mask is insufficient. The library currently doesn't support CJK natively, but you can romanise or use n‑gram tokens.
- **Dynamic alphabets**: The alphabet must be defined before building the dictionary. Changing it later requires rebuilding all masks.

## License

This project is licensed under the MIT License – see the [LICENSE](LICENSE) file for details.

## Acknowledgements

Inspired by similar techniques used in fuzzy finders like fuzzysort and FlashFuzzy. The bitmask idea is simple yet effective.

For a detailed explanation, see the [PAPER.md](PAPER.md) in the repository.

---

**Contributions and improvements are welcome!** If you polish the code, feel free to upload it to crates.io – just keep the original author credit.
```