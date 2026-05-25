---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [write_spec, render_spec]
---

@markdownai v1.0

# Skill-A Pack: write_spec / render_spec

@define write_spec(slug, body)

@set spec_date = "{{ @date format='YYYY-MM-DD' }}"
@set spec_path = "docs/mdai/specs/{{ spec_date }}-{{ slug }}-design.mdai.md"

@if file.exists "{{ spec_path }}"
- ABORT: Spec file already exists at {{ spec_path }}
- Choose a different slug, delete the existing file first, or amend the body in place.
- Not overwriting to prevent silent data loss.
@else
@mkdir docs/mdai/specs
@query mcp lean-ctx ctx_shell cmd="cat > \"{{ spec_path }}\" <<'SPEC_EOF'\n---\nslug: {{ slug }}\ndate: {{ spec_date }}\nstatus: ready-for-review\n---\n\n@markdownai v1.0\n\n# {{ slug }}\n\n{{ body }}\nSPEC_EOF"
- wrote {{ spec_path }}
@endif

@end

@define render_spec(slug, target)

@set spec_date = "{{ @date format='YYYY-MM-DD' }}"
@set spec_path = "docs/mdai/specs/{{ spec_date }}-{{ slug }}-design.mdai.md"

@switch target
  @case "none"
    # no-op
  @case "chat"
    @if file.exists "{{ spec_path }}"
      @query mcp markdownai read_file file="{{ spec_path }}"
    @else
      - err: Cannot render — spec file does not exist at {{ spec_path }}
      - Call write_spec(slug, body) first.
    @endif
  @case "file"
    @if file.exists "{{ spec_path }}"
      @mkdir docs/mdai/specs/rendered
      @query mcp lean-ctx ctx_shell cmd="cd markdownai && npx mai render \"../{{ spec_path }}\" > \"../docs/mdai/specs/rendered/{{ spec_date }}-{{ slug }}.rendered.md\""
    @else
      - err: Cannot render — spec file does not exist at {{ spec_path }}
      - Call write_spec(slug, body) first.
    @endif
@endswitch

@end
