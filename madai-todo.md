# MDAi Todos
- wieso macht ein agent 
  - lean-ctx - ctx_read (MCP)(path: "/home/tholo/Scripts/lean-ctx/rust/src/tools/registered/ctx_refactor.rs")
    danach lean-ctx - ctx_read (MCP)(path: "/home/tholo/Scripts/lean-ctx/rust/src/tools/registered/ctx_refactor.rs", mode: "full")
  Hier sollte der Cache doch greifen
- set `ctx_delta` if file cached, no fresh read in claude und für agents
- set path damit Pläne src path immer referzieren können
- muss in Macro alles aufgezählt sein? die calls passieren ja "einfach" so
- später: Wenn du Auto-Sync über alle Clients + Drift-Erkennung willst, könnten wir die Regeln per lean-ctx rules init in eine zentrale .leanctx/rules.toml heben und sync verteilen lässt. Größere Umstellung, für diese Aufgabe nicht erforderlich
- ctx_knowledge mit ctx_agent action=diary (ctx_agent.rs:500-519) auffüllen
- add 'lean-ctx mdai install'
    - setup markdownai
    - setup superpowers plugin
        - set up mdai skills
- add ci to update skill against superpowers skills
- rewrite markdownai to rust
    - add lean-ctx-mdai mcp tools
    - https://github.com/yuin/rushdown/tree/main für markdown parsen
- add mdai consumer renderer

## far away goals

- remove superpowers as a dependency
