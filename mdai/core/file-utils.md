---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [ file_check ]
---

@markdownai v1.0

@define file_check(path)
@if file.exists({{ path }})

- {{ path }} exists

@else

- {{ path }} MISSING

@if-end
@define-end
