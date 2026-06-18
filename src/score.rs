use strsim::jaro_winkler;

/// Returns the Jaro‑Winkler similarity between two strings (0.0 … 1.0).
pub fn similarity(source: &str, target: &str) -> f64 {
    jaro_winkler(source, target)
}