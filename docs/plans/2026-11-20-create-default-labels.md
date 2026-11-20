# `bd create` adds the default labels

Kind: implementation plan (bead `beads-rs-d9oz`), dated history once landed.

## Goal

`BD_LABEL=project:foo` (or `[defaults] labels`) scopes `ready`, `list`, `count`,
`blocked`, `stale` and `search` to `project:foo` (bead `beads-rs-o0bx`). `bd create` still
makes beads without that label, so a bead created in a scoped shell vanishes from the
scoped `bd ready` right away. Make `bd create` add the default's labels to new beads.

- The default's literal labels are added to every new bead: `bd create "x"` and each bead
  of `bd create -f plan.md`.
- They are added to explicit `-l/--labels`, not replaced by them: `bd create "x" -l bug`
  under `BD_LABEL=project:foo` gets `bug` and `project:foo`. (Filters replace the default
  because they narrow; create adds because the bead must stay visible in the scoped view.)
- Entries containing `*` are wildcards, not labels, and are skipped (a `*` would otherwise
  be stored as a literal label).
- `--no-default-labels` (existing global flag) turns it off for one command.
- Human output prints one stderr line naming the labels added and any wildcard skipped;
  `--json` is unchanged, as for the filter note.

## Non-goals

- A second variable or config key for create (`BD_CREATE_LABELS`): one default, one
  source of truth. Split it only if a real case needs different filter and create labels.
- Adding default labels on `bd update`, `bd label`, or other mutations.
- Any daemon or wire change: the CLI sends the labels in the existing `CreatePayload.labels`.

## Approach

- `crates/beads-cli/src/commands/create.rs`: one helper,
  `with_default_labels(ctx, labels: Vec<String>) -> Vec<String>`, appends each literal
  entry of `ctx.default_labels` not already present and prints the note in human mode.
  The single-create path and the markdown-file path both call it before building
  `CreatePayload`.
- Docs: `CLI_SPEC.md` default-label section and `bd prime` Labels section mention create.

## Reuse

`CliRuntimeCtx.default_labels` (already resolved from `BD_LABEL`/config and cleared by
`--no-default-labels`), `CreatePayload.labels`, and in tests `create_labeled_task` and
`test_default_label_filter_from_bd_label` in
`crates/beads-rs/tests/integration/cli/critical_path.rs`.

## Tests

- cli unit (`create.rs`): no default leaves labels as given; default appended after
  explicit labels without duplicates; wildcard entries skipped.
- integration: under `BD_LABEL=project:alpha,team:*`, `bd create` gives `project:alpha`,
  `-l bug` gives both, `--no-default-labels` gives none, and the new bead shows in the
  scoped `bd ready`; `-f` markdown creates get it too.
