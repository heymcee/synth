#!/usr/bin/env python3
"""Sequence optimization room using DNA Chisel (Edinburgh Genome Foundry).

Accepts a Seek frame (JSON) or FASTA. Returns an optimised frame or FASTA.
Optimised sequences intended for synthesis MUST still pass through bloc screen.

Example:
    python -m synth.optimize frame.json --host e_coli_k12 -o optimised.json
    python -m synth.optimize input.fasta --host yeast -o optimized.fasta
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

try:
    from dnachisel import (
        DnaOptimizationProblem,
        AvoidPattern,
        EnforceGCContent,
        CodonOptimize,
        EnforceTranslation,
    )
    from Bio import SeqIO
except ImportError as e:
    print("Install dependencies: pip install dnachisel biopython", file=sys.stderr)
    raise SystemExit(1) from e


# Map bloc chassis / common names → DNA Chisel species tables
HOST_MAP = {
    "e_coli_k12": "e_coli",
    "e_coli_b": "e_coli",
    "ecoli": "e_coli",
    "e_coli": "e_coli",
    "yeast": "s_cerevisiae",
    "s_cerevisiae": "s_cerevisiae",
    "human": "h_sapiens",
    "h_sapiens": "h_sapiens",
    "cell_free": "e_coli",
    "other_bsl1": "e_coli",
    "other": "e_coli",
    "none": "e_coli",
}


def resolve_species(host: str) -> str:
    key = host.lower().replace(" ", "_").replace("-", "_")
    return HOST_MAP.get(key, key)


def optimize_sequence(
    sequence: str,
    host: str = "e_coli",
    avoid_patterns: list[str] | None = None,
    gc_min: float = 0.3,
    gc_max: float = 0.7,
) -> str:
    """Optimise a coding sequence for the given host and synthesis constraints."""
    species = resolve_species(host)
    patterns = avoid_patterns or ["BsaI_site", "BsmBI_site"]
    constraints = [
        EnforceTranslation(),
        EnforceGCContent(mini=gc_min, maxi=gc_max),
    ]
    for p in patterns:
        constraints.append(AvoidPattern(p))

    problem = DnaOptimizationProblem(
        sequence=sequence.upper().replace("U", "T"),
        constraints=constraints,
        objectives=[CodonOptimize(species=species)],
    )
    problem.resolve_constraints()
    problem.optimize()
    return str(problem.sequence)


def optimize_frame(frame: dict[str, Any], host: str, **kwargs: Any) -> dict[str, Any]:
    """Optimise a Seek frame in place (returns a new dict)."""
    if frame.get("kind") != "DNA":
        raise ValueError("synth v0 optimises DNA frames only; use seek.filler for AA→DNA first")
    if frame.get("phase") not in ("filled", "spec"):
        raise ValueError(f"unexpected phase: {frame.get('phase')}")
    seq = frame.get("seq") or ""
    if not seq:
        raise ValueError("empty seq; phase should be 'spec' until filled")

    opt_seq = optimize_sequence(seq, host=host, **kwargs)
    out = dict(frame)
    out["seq"] = opt_seq
    out["phase"] = "filled"
    # Annotate that optimisation occurred
    features = list(out.get("features") or [])
    features.append({
        "type": "synth_optimised",
        "host": host,
        "species": resolve_species(host),
    })
    out["features"] = features
    return out


def main() -> None:
    parser = argparse.ArgumentParser(description="Optimise sequence for synthesis and host.")
    parser.add_argument("input", type=Path, help="Input Seek frame JSON or FASTA")
    parser.add_argument("-o", "--output", type=Path, help="Output path (same format as input by default)")
    parser.add_argument("--host", default="e_coli_k12", help="Target host / chassis")
    parser.add_argument("--gc-min", type=float, default=0.3)
    parser.add_argument("--gc-max", type=float, default=0.7)
    args = parser.parse_args()

    path = args.input
    text = path.read_text()
    is_json = path.suffix.lower() in {".json", ".seek"} or text.lstrip().startswith("{")

    if is_json:
        frame = json.loads(text)
        optimised = optimize_frame(
            frame, host=args.host, gc_min=args.gc_min, gc_max=args.gc_max
        )
        out_path = args.output or path.with_name(path.stem + "_opt.json")
        out_path.write_text(json.dumps(optimised, indent=2) + "\n")
        print(f"Wrote optimised Seek frame to {out_path}")
        print("Next: convert to bloc design specification and run bloc screen before any synthesis path.")
    else:
        records = list(SeqIO.parse(path, "fasta"))
        if not records:
            print("No sequences in input FASTA", file=sys.stderr)
            sys.exit(1)
        optimised_recs = []
        for rec in records:
            opt_seq = optimize_sequence(str(rec.seq), host=args.host, gc_min=args.gc_min, gc_max=args.gc_max)
            rec.seq = opt_seq  # type: ignore
            rec.id = rec.id + "_opt"
            rec.description = f"optimised for {args.host}"
            optimised_recs.append(rec)
        out_path = args.output or path.with_name(path.stem + "_opt.fasta")
        SeqIO.write(optimised_recs, out_path, "fasta")
        print(f"Wrote {len(optimised_recs)} optimised sequence(s) to {out_path}")
        print("Prefer Seek frames in Wetware sessions. Route through bloc before synthesis.")


if __name__ == "__main__":
    main()
