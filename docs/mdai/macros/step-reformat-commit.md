@markdownai v1.0

@define stepReformatCommit(file)

- `mcp__jetbrains__reformat_file {{ file }}`
- `git add {{ file }}`
- `git commit -m "..."` (Commit-Message gemäß Task-Vorgabe)
  @end
