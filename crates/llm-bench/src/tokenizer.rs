//! Tokenizer utilities

use crate::{BenchError, Result};
use tokenizers::Tokenizer;

pub fn load_tokenizer(path: &str) -> Result<Tokenizer> {
    Tokenizer::from_file(path)
        .map_err(|e| BenchError::Tokenizer(e.to_string()))
}

pub fn tokenize(tokenizer: &Tokenizer, text: &str, add_special: bool) -> Result<Vec<u32>> {
    let encoding = tokenizer.encode(text, add_special)
        .map_err(|e| BenchError::Tokenizer(e.to_string()))?;
    Ok(encoding.get_ids().to_vec())
}

pub fn detokenize(tokenizer: &Tokenizer, tokens: &[u32], skip_special: bool) -> Result<String> {
    tokenizer.decode(tokens, skip_special)
        .map_err(|e| BenchError::Tokenizer(e.to_string()))
}