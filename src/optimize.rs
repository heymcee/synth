//! Codon optimisation for synthesis and host.
//!
//! Method: most-frequent synonymous codon (local CAI maximisation),
//! with fallback to next synonymous codons to avoid forbidden patterns
//! and to keep GC in range where possible.

use crate::codon::{synonymous_codons, translate_codon};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizeError {
    #[error("sequence length not divisible by 3")]
    BadLength,
    #[error("unknown codon at position {0}")]
    UnknownCodon(usize),
    #[error("no synonymous codon avoids patterns at position {0}")]
    PatternConflict(usize),
}

#[derive(Debug, Clone)]
pub struct OptimizeOpts<'a> {
    pub host: &'a str,
    pub avoid: &'a [&'a str],
    pub gc_min: f64,
    pub gc_max: f64,
}

impl Default for OptimizeOpts<'static> {
    fn default() -> Self {
        Self {
            host: "e_coli",
            avoid: &["GGTCTC", "CGTCTC", "GAGACC", "GAGACG"], // BsaI / BsmBI approximate
            gc_min: 0.3,
            gc_max: 0.7,
        }
    }
}

/// Optimise a coding sequence (must be length % 3 == 0, DNA, uppercase).
pub fn optimize_cds(seq: &str, opts: &OptimizeOpts) -> Result<String, OptimizeError> {
    let seq = normalise(seq);
    if seq.len() % 3 != 0 {
        return Err(OptimizeError::BadLength);
    }

    let codons: Vec<&str> = seq
        .as_bytes()
        .chunks(3)
        .map(|c| std::str::from_utf8(c).unwrap_or("NNN"))
        .collect();

    let mut out = String::with_capacity(seq.len());

    for (i, codon) in codons.iter().enumerate() {
        let aa = translate_codon(codon).ok_or(OptimizeError::UnknownCodon(i * 3))?;
        let candidates = synonymous_codons(aa, opts.host);
        if candidates.is_empty() {
            return Err(OptimizeError::UnknownCodon(i * 3));
        }

        let chosen = choose_codon(&candidates, &out, opts.avoid)
            .ok_or(OptimizeError::PatternConflict(i * 3))?;
        out.push_str(chosen);
    }

    // Soft GC adjustment: if outside range, try swapping a few codons to alternatives
    // that move GC in the right direction without reintroducing patterns.
    if let Some(adjusted) = adjust_gc(&out, opts) {
        return Ok(adjusted);
    }

    Ok(out)
}

pub fn gc_content(seq: &str) -> f64 {
    if seq.is_empty() {
        return 0.0;
    }
    let gc = seq.chars().filter(|c| *c == 'G' || *c == 'C').count();
    gc as f64 / seq.len() as f64
}

fn normalise(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .map(|c| if c == 'U' { 'T' } else { c })
        .collect()
}

fn choose_codon(candidates: &[&str], prefix: &str, avoid: &[&str]) -> Option<&'static str> {
    for c in candidates {
        let trial = format!("{prefix}{c}");
        if !contains_any(&trial, avoid) {
            // Safety: candidates are 'static
            return Some(unsafe { std::mem::transmute::<&str, &'static str>(c) });
        }
    }
    None
}

fn contains_any(seq: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| seq.contains(p))
}

fn adjust_gc(seq: &str, opts: &OptimizeOpts) -> Option<String> {
    let gc = gc_content(seq);
    if gc >= opts.gc_min && gc <= opts.gc_max {
        return None;
    }
    let want_higher = gc < opts.gc_min;
    let mut codons: Vec<String> = seq
        .as_bytes()
        .chunks(3)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect();

    for i in 0..codons.len() {
        let aa = translate_codon(&codons[i])?;
        let alts = synonymous_codons(aa, opts.host);
        for alt in alts {
            if alt == codons[i] {
                continue;
            }
            let alt_gc = alt.chars().filter(|c| *c == 'G' || *c == 'C').count();
            let cur_gc = codons[i].chars().filter(|c| *c == 'G' || *c == 'C').count();
            let improves = if want_higher {
                alt_gc > cur_gc
            } else {
                alt_gc < cur_gc
            };
            if !improves {
                continue;
            }
            let old = codons[i].clone();
            codons[i] = alt.to_string();
            let trial: String = codons.iter().cloned().collect();
            if !contains_any(&trial, opts.avoid) {
                let new_gc = gc_content(&trial);
                if new_gc >= opts.gc_min && new_gc <= opts.gc_max {
                    return Some(trial);
                }
                // keep the improvement and continue
            } else {
                codons[i] = old;
            }
        }
    }
    let final_seq: String = codons.into_iter().collect();
    let final_gc = gc_content(&final_seq);
    if final_gc >= opts.gc_min && final_gc <= opts.gc_max {
        Some(final_seq)
    } else {
        None
    }
}
