use clap::Args;

use super::common::fmt_wall_ms;
use super::{CommandResult, print_ok};
use crate::runtime::{CliRuntimeCtx, send};
use crate::validation::{normalize_bead_id, validation_error};
use beads_api::{FieldHistory, IssueField};
use beads_surface::ipc::{FieldHistoryPayload, Request};

#[derive(Args, Debug)]
pub struct LogArgs {
    /// Issue ID whose journal should be inspected.
    pub id: String,

    /// Mutable field to follow through the journal.
    #[arg(long, value_name = "NAME", required = true)]
    pub field: String,
}

pub fn handle(ctx: &CliRuntimeCtx, args: LogArgs) -> CommandResult<()> {
    let id = normalize_bead_id(&args.id)?;
    let field = IssueField::parse(&args.field).ok_or_else(|| {
        validation_error(
            "field",
            format!(
                "unknown field {:?}; expected one of: {}",
                args.field,
                IssueField::VALID_NAMES
            ),
        )
    })?;
    let req = Request::FieldHistory {
        ctx: ctx.read_ctx(),
        payload: FieldHistoryPayload { id, field },
    };
    let ok = send(&req)?;
    print_ok(&ok, ctx.json)
}

pub fn render_field_history(history: &FieldHistory) -> String {
    if history.entries.is_empty() {
        return format!(
            "No journal history for {} field {}",
            history.id,
            history.field.as_str()
        );
    }

    let mut out = String::new();
    for entry in &history.entries {
        use std::fmt::Write as _;

        let reason = entry
            .reason
            .as_deref()
            .map(|reason| format!(" reason={reason:?}"))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "{} {} {}={}{} [{}:{}]",
            fmt_wall_ms(entry.at.wall_ms),
            entry.by,
            history.field.as_str(),
            entry.value,
            reason,
            entry.origin_replica_id,
            entry.origin_seq,
        );
    }
    out.trim_end().to_owned()
}
