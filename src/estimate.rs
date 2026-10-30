//! A fast, dependency-free token estimate.
//!
//! This is a heuristic, not a real tokenizer. It is shaped after how BPE
//! tokenizers used by today's LLMs treat source code: words are split into
//! chunks of a few characters, most punctuation is its own token, runs of
//! indentation collapse, and non-ASCII letters cost more per character.

/// Estimate how many tokens `text` costs an LLM.
pub fn estimate_tokens(text: &str) -> u64 {
    let mut tokens: u64 = 0;
    let mut ascii_word: u64 = 0;
    let mut ws_run: u64 = 0;
    let mut ws_has_newline = false;

    let flush_word = |n: &mut u64, t: &mut u64| {
        if *n > 0 {
            *t += (*n).div_ceil(4);
            *n = 0;
        }
    };

    for c in text.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if ws_run > 0 {
                tokens += ws_cost(ws_run, ws_has_newline);
                ws_run = 0;
                ws_has_newline = false;
            }
            ascii_word += 1;
        } else if c.is_whitespace() {
            flush_word(&mut ascii_word, &mut tokens);
            ws_run += 1;
            if c == '\n' {
                ws_has_newline = true;
            }
        } else {
            flush_word(&mut ascii_word, &mut tokens);
            if ws_run > 0 {
                tokens += ws_cost(ws_run, ws_has_newline);
                ws_run = 0;
                ws_has_newline = false;
            }
            tokens += 1;
        }
    }
    flush_word(&mut ascii_word, &mut tokens);
    if ws_run > 0 {
        tokens += ws_cost(ws_run, ws_has_newline);
    }
    tokens
}

/// A single space usually merges into the next word; longer runs and line
/// breaks (with their indentation) cost about one token.
fn ws_cost(run: u64, has_newline: bool) -> u64 {
    if has_newline || run > 1 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn short_words_are_one_token_each() {
        assert_eq!(estimate_tokens("let x be"), 3);
    }

    #[test]
    fn long_identifiers_split() {
        // 24 chars -> 6 chunks
        assert_eq!(estimate_tokens("averyveryverylongidentif"), 6);
    }

    #[test]
    fn punctuation_counts() {
        assert_eq!(estimate_tokens("f(a, b);"), 7);
    }

    #[test]
    fn indentation_collapses() {
        let a = estimate_tokens("x\n        y");
        let b = estimate_tokens("x\ny");
        assert_eq!(a, b);
    }

    #[test]
    fn non_ascii_costs_more() {
        assert!(estimate_tokens("ığüşöç") > estimate_tokens("igusoc"));
    }
}
