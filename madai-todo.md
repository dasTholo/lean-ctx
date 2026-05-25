# MDAi Todos

- muss in Macro alles aufgezählt sein? die calls passieren ja "einfach" so
- ctx_shell (MCP)(command: "test -f /home/tholo/Scripts/lean-ctx/mdai/skills/mdai-brainstorm/body.mdai.md && echo
  EXISTS || echo MISSING") abfangen mit search
- awk wc abfangen
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
