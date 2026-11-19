# Label filters for `bd blocked`, `bd stale` and `bd search`

Kind: implementation plan (bead `beads-rs-82nf`), dated history once landed.

## Goal

In a monorepo, labels scope beads to projects (`project:<name>`). `bd list`, `bd count`
and `bd ready` already take `-l/--label` (ALL) and `--label-any` (AT LEAST ONE), with `*`
as wildcard. `bd blocked`, `bd stale` and `bd search` do not, so their output always
covers the whole repository. Give these three the same flags with the same semantics.

## Non-goals

- `bd status` and `bd epic status` label scoping (summary counts; separate question).
- A default label filter from the environment (`BD_LABEL`): separate bead, designed after
  this lands, so it can apply to one set of commands that all accept the flags.
- `bd create` adding a project label automatically.
- Any change to label matching itself (`Filters::matches`, `label_matches`).

## Approach

One label-filter shape on the wire, one on the command line, one matcher in the daemon.

- **beads-surface** (`ipc/payload.rs`): add `LabelFilter { labels, labels_any }`
  (both `Option<Vec<String>>`, serde default, skipped when `None`) with
  `to_filters() -> Option<Filters>`, moved out of `ReadyPayload::label_filters()`.
  `ReadyPayload` flattens it in place of its two fields (JSON shape unchanged).
  `StalePayload` flattens it. New `BlockedPayload { #[serde(flatten)] label_filter }`
  replaces `EmptyPayload` in `Request::Blocked` (`ipc/types.rs`); an old client's
  empty payload still decodes.
- **beads-daemon** (`runtime/query_executor.rs`, `runtime/coord/request.rs`):
  `blocked_for_namespace` and `query_stale` take `Option<&Filters>` and skip beads
  whose view fails `Filters::matches` — the code `list` and `ready` use. Stale filters
  before the limit, as `ready` does.
- **beads-cli**: `Command::Blocked` becomes `Blocked(BlockedArgs)`; `BlockedArgs`,
  `StaleArgs` and `SearchArgs` flatten the existing `LabelFilterArgs`. `search` copies the
  labels into its `Filters` (it already sends `Request::List`, so no daemon change).
  `LabelFilterArgs` gains `to_payload() -> LabelFilter` so `ready`, `blocked` and `stale`
  build the payload the same way.
- **CLI_SPEC.md**: list the new flags under `blocked`, `stale`, `search`.

## Reuse

`LabelFilterArgs` (flags, help text), `Filters::matches` / `label_matches` (matching,
wildcards), the `labeled_ready_store` fixture in `query_executor.rs` tests, and
`test_ready_label_filters` in `crates/beads-rs/tests/integration/cli/critical_path.rs`.

## Files

- `crates/beads-surface/src/ipc/payload.rs`, `crates/beads-surface/src/ipc/types.rs`
- `crates/beads-daemon/src/runtime/query_executor.rs`, `crates/beads-daemon/src/runtime/coord/request.rs`
- `crates/beads-cli/src/filters.rs`, `crates/beads-cli/src/cli.rs`,
  `crates/beads-cli/src/commands/{blocked,stale,search,ready}.rs`
- `crates/beads-rs/tests/integration/cli/critical_path.rs`, `CLI_SPEC.md`

## Tests

- surface: `BlockedPayload` decodes from `{}`; `ReadyPayload` JSON with labels is
  byte-identical before and after the flatten.
- daemon unit: blocked and stale with AND / OR label filters on `labeled_ready_store`
  (stale: filter applies before the limit).
- integration: `bd blocked -l`, `bd stale -d 0 -l`, `bd search <q> -l` each return only
  the labelled beads; without `-l` output is unchanged.
