# ProjectAtlas workflow

- Use ProjectAtlas repository intelligence before broad filesystem searches or full-file reads.
- When ProjectAtlas MCP tools are available, call `atlas_session_brief` once with the current task and compact output, then follow its returned next call. Initialize with `atlas_init` only when the project atlas is absent, and refresh it only when it may be stale.
- With the CLI fallback, narrow through `projectatlas overview`, `projectatlas folders`, and `projectatlas files`; then use `summary`, `outline`, `search`, or `slice` for the selected file.
- After repository changes, refresh with `projectatlas watch --once` and validate with `projectatlas lint --report-untracked --purpose-level low`.
- Treat `.projectatlas/projectatlas.db` as authoritative local state. Update purposes through `projectatlas purpose set` or `projectatlas purpose review`; never edit the database directly.
