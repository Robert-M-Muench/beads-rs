# Default label filter (`BD_LABEL`)

Kind: implementation plan (bead `beads-rs-o0bx`), dated history once landed.

## Goal

In a monorepo each project's beads carry a label such as `project:3d-sim`. Typing
`-l project:3d-sim` on every `bd ready` is noise and easy to forget. Let a shell (or an
agent's environment) set a default label filter once, and let any single command
ignore it.

- `BD_LABEL=project:3d-sim` (comma-separated, `*` wildcard as for `-l`) is the default
  for `-l/--label` on the commands that list beads: `ready`, `list`, `count`, `blocked`,
  `stale`, `search`.
- The same default can live in config as `[defaults] labels = [...]` (user
  `config.toml` or repo `beads.toml`); `BD_LABEL` overrides it, and `BD_LABEL=` (empty)
  clears it.
- Explicit label flags replace the default instead of adding to it: `-l`, `--label-any`,
  and on `list`/`count` `--no-labels`.
- `--no-default-labels` (global flag) ignores the default for one command.
- Human output prints one stderr line naming the active default, so filtered-out beads
  never look lost. `--json` output is unchanged (stdout stays machine-clean).

## Non-goals

- `bd` reading `.env` files itself. Per-directory values come from the shell (direnv
  `.envrc`, which can `dotenv` a `.env`) or the agent's settings `env`.
- Path-scoped defaults inside `beads.toml` (label per subdirectory).
- `bd create` adding the default label to new beads (separate bead if wanted).
- `status`, `epic status`, `show`, and mutations: unaffected.
- Any daemon or wire change: the default is resolved in the CLI and sent as ordinary
  label filters, because one daemon serves many shells.

## Approach

- **beads-bootstrap** (`config/schema.rs`, `config/env.rs`, `config/merge.rs`):
  `DefaultsConfig.labels: Option<Vec<String>>`, merged like the other defaults.
  `LABEL_VAR = "BD_LABEL"` joins `ENV_OVERRIDE_KEYS` and `CONFIG_ENV_KEYS` (so the
  integration fixtures scrub it); the override splits on `,`, trims, drops empties, and
  an empty result clears the default.
- **beads-cli**:
  - `cli.rs`: global `--no-default-labels`; `RuntimeBuildArgs` carries it.
  - `runtime.rs`: `CliRuntimeCtx.default_labels: Option<Vec<String>>` — the resolved
    default, `None` when unset or turned off. Runtime already owns env/config-derived
    context, so commands never read the environment.
  - `filters.rs`: `LabelFilterArgs::resolve(&self, ctx) -> LabelFilter` returns the
    explicit flags when any are given, otherwise the default as `labels`, and prints the
    stderr note in human mode when the default was used. `ready`, `blocked`, `stale`
    and `search` call it in place of `to_payload()`/`labels()`; `apply_common_filters`
    (list, count) calls it too and treats `--no-labels` as explicit.
- **beads-rs** (`src/cli/mod.rs`): `build_runtime` fills `default_labels` from
  `config.defaults.labels` unless `--no-default-labels`.
- **Docs**: `CLI_SPEC.md` (global flag, env var), the `bd prime` / help text where
  label flags are described if they mention env vars.

## Reuse

`LabelFilterArgs`, `LabelFilter`, `Filters::matches` (unchanged matching), the config
layer/env-override machinery, `CONFIG_ENV_KEYS` scrubbing in
`crates/beads-rs/tests/integration/fixtures/bd_runtime.rs`, and the label integration
tests in `crates/beads-rs/tests/integration/cli/critical_path.rs`.

## Tests

- bootstrap: `BD_LABEL` parses comma lists, trims, empty clears a config-file default;
  `[defaults] labels` merges repo over user.
- cli unit: `resolve` uses explicit flags over the default, the default when no flag,
  nothing when no default.
- integration: with `BD_LABEL` set, `ready`/`list`/`count`/`blocked`/`stale`/`search`
  are scoped; `-l other` replaces it; `--no-default-labels` and `BD_LABEL=` show all;
  human `ready` prints the stderr note, `--json` stdout has no note.
