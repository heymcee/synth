# synth

**Design studio / sequence optimization room.**

For biodesign agent use. Optimises sequence parameters for synthesis and host.

- **Type**: design studio / environment
- **Purpose**: sequence optimization room
- **INPUT**: sequence `.fasta` FROM sequence card
- **PROCESS**: OPTIMISE sequence parameters FOR ( synthesis, host )
- **OUTPUT**: optimised sequence `.fasta`

Uses tools from the [Edinburgh Genome Foundry](https://github.com/Edinburgh-Genome-Foundry), primarily [DNA Chisel](https://github.com/Edinburgh-Genome-Foundry/DnaChisel) for versatile sequence optimization (codon optimization, GC content, restriction site avoidance, synthesis constraints, etc.).

## Status

v0.1 skeleton. License: MIT.

## Intended use

Agents in Wetware sessions pass a sequence card (FASTA) into this room. The room returns an optimized FASTA ready for synthesis provider constraints and host expression.

Wet-laboratory execution is out of process. Always route final designs through a biosafety gate such as [bloc](https://github.com/wetware-hq/bloc) before ordering.

## Quick start

```bash
pip install dnachisel biopython
python -m synth optimize input.fasta --host ecoli --output optimized.fasta
```

## References

- DNA Chisel: https://edinburgh-genome-foundry.github.io/DnaChisel/
- EGF Codons suite: https://edinburgh-genome-foundry.github.io/
