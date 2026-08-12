# Triage Gold Validation Set

`triage_gold_set.jsonl` contains one JSON object per task: `id`, `query`, `language`,
`labels`, and `metadata`. Labels define `intent`, `task_class`, `complexity`, `scope`,
`reasoning_need`, and `risk`; metadata identifies synthetic provenance and annotation.

- Intent: generate, coding_fix, refactor, explore, test, debug, config, deploy, review.
- Complexity: mechanical (local/repetitive), standard (bounded implementation), architectural
  (system design). Scope: single_file, multi_file, cross_module, cross_project.
- Distribution: 500 tasks; 304 English / 146 German / 25 French / 25 Spanish; intents
  44--69 each; complexity 99 mechanical / 275 standard / 126 architectural; scope
  119 single-file / 190 multi-file / 111 cross-module / 80 cross-project respectively.
- The expanded set includes security reviews, performance investigation, API design,
  documentation, deployment, data-migration, multi-intent, and deliberately vague tasks.
