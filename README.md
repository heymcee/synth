# synth

**Design studio / sequence optimization room.**

Systems engineering in **Rust**. Optimises Seek frames for synthesis constraints and host expression.

| Item | Value |
|------|--------|
| Type | design studio / environment |
| Purpose | sequence optimization room |
| Language | Rust (systems) |
| Input | Seek frame (preferred) or sequence `.fasta` |
| Process | OPTIMISE sequence parameters FOR (synthesis, host) |
| Output | optimised Seek frame or `.fasta` |
| Method | Most-frequent synonymous codon (local CAI) + pattern avoidance + GC bounds |
| Gate | Final designs **must** pass [bloc](https://github.com/wetware-hq/bloc) before synthesis |

## Organization tools

| Repo | Role | Language |
|------|------|----------|
| [seek](https://github.com/wetware-hq/seek) | Canonical sequence representation (frame JSON) | Python |
| **synth** (this repo) | Optimise frame for host + synthesis constraints | **Rust** |
| [bloc](https://github.com/wetware-hq/bloc) | Biosecurity suppressor; release policy | Rust |
| [bindi](https://github.com/wetware-hq/bindi) | Protein binder design / curation | Python |
| [captaincook](https://github.com/wetware-hq/captaincook) | Wetware Sydney head engineer | Python |

Pipeline:

```
seek frame (filled)  →  synth  →  optimised frame
                              ↓
                    bloc design specification  →  bloc screen  →  card
```

synth does **not** set release policy.

## Quick start

```bash
cargo build --release
./target/release/synth examples/gfp_frame.json --host e_coli_k12 -o optimised.json
./target/release/synth input.fasta --host yeast -o optimized.fasta
```

Default: most-frequent codon for the host, avoid BsaI/BsmBI sites (`GGTCTC`, `GAGACC`, `CGTCTC`, `GAGACG`), GC 0.3–0.7, preserve translation (standard code).

Chassis mapping: `e_coli_k12` / `e_coli_b` → `e_coli`, `yeast` → `s_cerevisiae`, `human` → `h_sapiens`.

## Layout

```
src/
  main.rs       CLI (Seek frame or FASTA)
  codon.rs      genetic code + preferred codon tables
  optimize.rs   CAI-style optimisation + pattern/GC handling
schema/
examples/
AGENTS.md       host contract
```

A Python DNA Chisel prototype remains under `synth/` for reference; the systems path is the Rust binary.

## Status

v0.1. License: MIT. Wet-laboratory execution is out of process.

## References

- Seek: https://github.com/wetware-hq/seek
- bloc: https://github.com/wetware-hq/bloc
- EGF DNA Chisel (design reference): https://github.com/Edinburgh-Genome-Foundry/DnaChisel
