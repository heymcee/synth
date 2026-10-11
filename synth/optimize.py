#!/usr/bin/env python3
"""Sequence optimization using DNA Chisel (Edinburgh Genome Foundry).

Example:
    python -m synth.optimize input.fasta --host ecoli -o optimized.fasta
"""

import argparse
import sys
from pathlib import Path

try:
    from dnachisel import DnaOptimizationProblem, AvoidPattern, EnforceGCContent, CodonOptimize, EnforceTranslation
    from Bio import SeqIO
except ImportError as e:
    print("Install dependencies: pip install dnachisel biopython", file=sys.stderr)
    raise SystemExit(1) from e


def optimize_sequence(sequence: str, host: str = "ecoli") -> str:
    """Optimise a coding sequence for the given host and basic synthesis constraints."""
    # Assume sequence is a CDS; enforce translation and codon-optimise.
    problem = DnaOptimizationProblem(
        sequence=sequence,
        constraints=[
            EnforceTranslation(),
            AvoidPattern("BsaI_site"),
            AvoidPattern("BsmBI_site"),
            EnforceGCContent(mini=0.3, maxi=0.7),
        ],
        objectives=[
            CodonOptimize(species=host),
        ],
    )
    problem.resolve_constraints()
    problem.optimize()
    return str(problem.sequence)


def main():
    parser = argparse.ArgumentParser(description="Optimise sequence for synthesis and host.")
    parser.add_argument("input", type=Path, help="Input FASTA file")
    parser.add_argument("-o", "--output", type=Path, default=Path("optimized.fasta"), help="Output FASTA")
    parser.add_argument("--host", default="ecoli", help="Target host species for codon optimization (e.g. ecoli, human, yeast)")
    args = parser.parse_args()

    records = list(SeqIO.parse(args.input, "fasta"))
    if not records:
        print("No sequences in input FASTA", file=sys.stderr)
        sys.exit(1)

    optimized = []
    for rec in records:
        opt_seq = optimize_sequence(str(rec.seq), host=args.host)
        rec.seq = opt_seq
        rec.id = rec.id + "_opt"
        rec.description = f"optimised for {args.host}"
        optimized.append(rec)

    SeqIO.write(optimized, args.output, "fasta")
    print(f"Wrote {len(optimized)} optimised sequence(s) to {args.output}")


if __name__ == "__main__":
    main()
