# synth

**Design studio / sequence optimization room.**

For biodesign agent use. Optimises sequence parameters for synthesis constraints and host expression.

| Item | Value |
|------|--------|
| Type | design studio / environment |
| Purpose | sequence optimization room |
| Input | Seek frame (preferred) or sequence `.fasta` from sequence card |
| Process | OPTIMISE sequence parameters FOR (synthesis, host) |
| Output | optimised Seek frame or `.fasta` |
| Engine | [DNA Chisel](https://github.com/Edinburgh-Genome-Foundry/DnaChisel) (Edinburgh Genome Foundry) |
| Gate | Final designs **must** pass [bloc](https://github.com/wetware-hq/bloc) before synthesis |

## Organization tools

| Repo | Role |
|------|------|
| [seek](https://github.com/wetware-hq/seek) | Canonical sequence representation (frame JSON) |
| **synth** (this repo) | Optimise frame for host + synthesis constraints |
| [bloc](https://github.com/wetware-hq/bloc) | Biosecurity suppressor; release policy |
| [bindi](https://github.com/wetware-hq/bindi) | Protein binder design / curation |
| [captaincook](https://github.com/wetware-hq/captaincook) | Wetware Sydney head engineer (structure, design, clinical context) |

Pipeline for a coding sequence intended for synthesis:

```
seek frame (filled)  →  synth  →  optimised frame
                              ↓
                    bloc design specification  →  bloc screen  →  card
```

synth does **not** set release policy. Absence of a bloc `RELEASE` is not clearance.

## Agent contract

See [AGENTS.md](AGENTS.md). Hosts must treat Seek frames as the primary sequence card and must route any synthesis path through `bloc screen`.

## Quick start

```bash
pip install -e .
# or: pip install dnachisel biopython

# Seek frame (preferred)
python -m synth.optimize examples/gfp_frame.json --host e_coli_k12 -o optimised.json

# FASTA (convenience)
python -m synth.optimize input.fasta --host yeast -o optimized.fasta
```

Default problem: `EnforceTranslation` + avoid BsaI/BsmBI + GC 0.3–0.7 + `CodonOptimize` for the host.

Chassis mapping (bloc → DNA Chisel): `e_coli_k12`/`e_coli_b` → `e_coli`, `yeast` → `s_cerevisiae`, `human` → `h_sapiens`.

## Schema

- Request: [`schema/optimize_request.schema.json`](schema/optimize_request.schema.json)
- Frame: https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json

## Status

v0.1 skeleton. License: MIT.

Wet-laboratory execution is out of process.

## References

- DNA Chisel: https://edinburgh-genome-foundry.github.io/DnaChisel/
- EGF Codons suite: https://edinburgh-genome-foundry.github.io/
- Seek: https://github.com/wetware-hq/seek
- bloc: https://github.com/wetware-hq/bloc
