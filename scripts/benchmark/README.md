# Token Reduction Benchmark

From the repository root, run:

```bash
scripts/benchmark/token_reduction.sh
```

The script runs ten fixed repository operations once without LeanCTX and once
with LeanCTX. Raw operations use `LEAN_CTX_DISABLED=1` plus native `cat`,
`rg`, `find`, or shell commands; compressed operations use the matching
`lean-ctx` CLI command. Captured outputs are stored under `results/latest/`
with summary data in `results/token_reduction.tsv` and `results/report.md`.

Generate the Markdown report again with:

```bash
scripts/benchmark/report.sh
```

“Tokens” are a deliberately conservative approximation: output characters
divided by four (`chars / 4`, integer-truncated). This measures emitted input
size, not model-specific tokenizer output. Reduction is calculated from the
per-task token estimates; the report's final row is the arithmetic mean across
the ten tasks.
