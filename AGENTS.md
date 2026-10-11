# Agent contract

Normative requirements for hosts and agents that use synth. Wording uses RFC 2119: **MUST**, **MUST NOT**, **SHOULD**.

## Scope

synth is a design-studio room that optimises sequence parameters for synthesis constraints and host expression. It does **not** set release policy.

The canonical sequence representation is a [Seek frame](https://github.com/wetware-hq/seek) (`schema/frame.schema.json`). The only sequence-bearing object that may proceed toward ordering, assembly, or transformation is a [bloc design specification](https://github.com/wetware-hq/bloc) that has received a `RELEASE` verdict from `bloc screen`.

## Interface

```
seek frame (phase: filled)  →  synth optimise  →  optimised seek frame
                                              ↓
                                    bloc design specification  →  bloc screen  →  Verdict + card
```

Hosts **MUST** route any optimised sequence that is intended for synthesis through `bloc screen` before it leaves the local machine. synth **MUST NOT** be the sole basis for a synthesis clearance.

## Inputs

- Prefer a Seek frame JSON (required fields: `id`, `kind`, `topo`, `seq`, `features`, `want`, `forbid`, `phase`).
- `kind` **MUST** be `DNA` for codon optimisation of coding sequences.
- `phase` **SHOULD** be `filled` (non-empty `seq`).
- FASTA is accepted as a convenience input; the host **SHOULD** convert it to a Seek frame before calling synth in a Wetware session.
- Host / chassis names **SHOULD** match bloc `chassis` values where possible (`e_coli_k12`, `e_coli_b`, `yeast`, …). synth maps these to DNA Chisel species tables.

## Outputs

- Optimised Seek frame (same schema, updated `seq`, phase remains `filled`, optional note in description or features).
- Optimised FASTA (legacy / interchange).
- Optimisation report (DNA Chisel summary) **SHOULD** be retained for traceability.

## Constraints and objectives (v0)

Default problem (DNA Chisel):

- `EnforceTranslation` (preserve amino-acid sequence for CDS)
- `AvoidPattern` for common Type IIS sites used in Golden Gate (BsaI, BsmBI)
- `EnforceGCContent` (0.3–0.7)
- Objective: `CodonOptimize` for the target host

Additional synthesis-provider constraints or custom objectives **MAY** be supplied; they **MUST** be declared in the request so the report remains auditable.

## Out of scope

- Release / HOLD / ESCALATE decisions (bloc owns these)
- Wet-laboratory execution
- Sequences-of-concern screening (commec lives in bloc)
- Inventing DNA from amino-acid sequences without an explicit fill step (use `seek.filler` first)

## Status

v0.1. Aligns with Seek frame v0 and bloc design_spec 0.1.0.
