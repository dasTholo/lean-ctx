---
lib_version: 0.1.1
mdai-pack:
  mode: import-only
  exports: [write_spec, render_spec, write_review_report]
---

@markdownai v1.0

# Skill-A Pack: write_spec / render_spec

@define write_spec(slug, body)

@if file.exists "docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"

- ABORT: Spec file already exists at docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md
- Choose a different slug, delete the existing file first, or amend the body in place.
- Not overwriting to prevent silent data loss.
  @else
  @mkdir docs/mdai/specs /
  @render-template from="mdai/skills/mdai-brainstorm/templates/spec-template.md" to="docs/mdai/specs/{{ @date
  format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md" force
  slug={{ slug }}
  date={{ @date format='YYYY-MM-DD' }}
  body={{ body }}
  @render-template-end
- wrote docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md
  @if-end

@define-end

@define render_spec(slug, target)

@set spec_date = "{{ @date format='YYYY-MM-DD' }}" /
@set spec_path = "docs/mdai/specs/{{ spec_date }}-{{ slug }}-design.mdai.md" /

@switch target
@case "none"

# no-op

@case "chat"
@if file.exists "{{ spec_path }}"
@query mcp markdownai read_file file="{{ spec_path }}" /
@else

- err: Cannot render — spec file does not exist at {{ spec_path }}
- Call write_spec(slug, body) first.
  @if-end
  @case "file"
  @if file.exists "{{ spec_path }}"
  @mkdir docs/mdai/specs/rendered /
  @query mcp lean-ctx ctx_shell cmd="cd markdownai && npx mai render \"../{{ spec_path }}\" > /
  \"../docs/mdai/specs/rendered/{{ spec_date }}-{{ slug }}.rendered.md\""
  @else
- err: Cannot render — spec file does not exist at {{ spec_path }}
- Call write_spec(slug, body) first.
  @if-end
  @switch-end

@define-end

@define write_review_report(slug, spec_path, date, status, strengths, issues, recommendations)
@mkdir docs/mdai/reviews /
@render-template from="mdai/skills/mdai-brainstorm/templates/review-template.md" to="docs/mdai/reviews/{{ slug
}}-review.md" force
spec_path={{ spec_path }}
date={{ date }}
status={{ status }}
strengths={{ strengths }}
issues={{ issues }}
recommendations={{ recommendations }}
@render-template-end

- wrote docs/mdai/reviews/{{ slug }}-review.md
  @define-end
