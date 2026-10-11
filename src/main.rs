//! synth — sequence optimization room.
//!
//! Optimises Seek frames (or FASTA) for synthesis constraints and host.
//! Does not set release policy. Route through bloc before synthesis.

mod codon;
mod optimize;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "synth", version, about = "Optimise sequence for synthesis and host")]
struct Args {
    /// Input Seek frame JSON or FASTA
    input: PathBuf,

    /// Output path (defaults to <input>_opt.<ext>)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Target host / chassis (e_coli_k12, yeast, human, …)
    #[arg(long, default_value = "e_coli_k12")]
    host: String,

    /// Patterns to avoid (repeatable). Default: BsaI / BsmBI sites.
    #[arg(long)]
    avoid: Vec<String>,

    /// Minimum GC fraction
    #[arg(long, default_value_t = 0.3)]
    gc_min: f64,

    /// Maximum GC fraction
    #[arg(long, default_value_t = 0.7)]
    gc_max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Frame {
    id: String,
    kind: String,
    topo: String,
    seq: String,
    #[serde(default)]
    features: Vec<serde_json::Value>,
    #[serde(default)]
    want: Vec<String>,
    #[serde(default)]
    forbid: Vec<String>,
    phase: String,
}

fn resolve_host(h: &str) -> String {
    let key = h.to_lowercase().replace([' ', '-'], "_");
    match key.as_str() {
        "e_coli_k12" | "e_coli_b" | "ecoli" | "e_coli" | "cell_free" | "other_bsl1" | "other" | "none" => "e_coli".into(),
        "yeast" | "s_cerevisiae" => "s_cerevisiae".into(),
        "human" | "h_sapiens" => "h_sapiens".into(),
        other => other.to_string(),
    }
}

fn default_avoid() -> Vec<String> {
    // BsaI: GGTCTC / GAGACC ; BsmBI: CGTCTC / GAGACG
    vec![
        "GGTCTC".into(),
        "GAGACC".into(),
        "CGTCTC".into(),
        "GAGACG".into(),
    ]
}

fn main() {
    let args = Args::parse();
    let text = fs::read_to_string(&args.input).expect("read input");
    let is_json = args.input.extension().map(|e| e == "json" || e == "seek").unwrap_or(false)
        || text.trim_start().starts_with('{');

    let host = resolve_host(&args.host);
    let avoid: Vec<String> = if args.avoid.is_empty() {
        default_avoid()
    } else {
        args.avoid
    };
    let avoid_refs: Vec<&str> = avoid.iter().map(|s| s.as_str()).collect();

    let opts = optimize::OptimizeOpts {
        host: &host,
        avoid: &avoid_refs,
        gc_min: args.gc_min,
        gc_max: args.gc_max,
    };

    if is_json {
        let mut frame: Frame = serde_json::from_str(&text).expect("parse Seek frame");
        if frame.kind != "DNA" {
            eprintln!("synth v0 optimises DNA frames only");
            std::process::exit(2);
        }
        if frame.seq.is_empty() {
            eprintln!("empty seq");
            std::process::exit(2);
        }
        let opt = optimize::optimize_cds(&frame.seq, &opts).expect("optimise");
        let gc = optimize::gc_content(&opt);
        frame.seq = opt;
        frame.phase = "filled".into();
        frame.features.push(serde_json::json!({
            "type": "synth_optimised",
            "host": args.host,
            "species": host,
            "gc": gc,
        }));

        let out = args.output.unwrap_or_else(|| {
            let mut p = args.input.clone();
            let stem = p.file_stem().unwrap().to_string_lossy();
            p.set_file_name(format!("{stem}_opt.json"));
            p
        });
        fs::write(&out, serde_json::to_string_pretty(&frame).unwrap() + "\n").expect("write");
        println!("Wrote optimised Seek frame to {}", out.display());
        println!("GC={:.3}. Next: bloc design specification → bloc screen before any synthesis path.", gc);
    } else {
        // Simple FASTA (single or multi)
        let mut records = parse_fasta(&text);
        if records.is_empty() {
            eprintln!("no sequences in FASTA");
            std::process::exit(1);
        }
        for (id, seq) in records.iter_mut() {
            let opt = optimize::optimize_cds(seq, &opts).expect("optimise");
            *seq = opt;
            *id = format!("{id}_opt");
        }
        let out = args.output.unwrap_or_else(|| {
            let mut p = args.input.clone();
            let stem = p.file_stem().unwrap().to_string_lossy();
            p.set_file_name(format!("{stem}_opt.fasta"));
            p
        });
        let mut buf = String::new();
        for (id, seq) in &records {
            buf.push('>');
            buf.push_str(id);
            buf.push('\n');
            buf.push_str(seq);
            buf.push('\n');
        }
        fs::write(&out, buf).expect("write");
        println!("Wrote {} optimised sequence(s) to {}", records.len(), out.display());
        println!("Prefer Seek frames in Wetware sessions. Route through bloc before synthesis.");
    }
}

fn parse_fasta(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut id = String::new();
    let mut seq = String::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('>') {
            if !id.is_empty() {
                out.push((id.clone(), seq.clone()));
            }
            id = rest.split_whitespace().next().unwrap_or("seq").to_string();
            seq.clear();
        } else {
            seq.push_str(line.trim());
        }
    }
    if !id.is_empty() {
        out.push((id, seq));
    }
    out
}
