use beads_surface::ipc::{BlockedPayload, Request};
use clap::Args;

use super::common::fmt_issue_ref;
use super::{CommandResult, print_ok};
use crate::filters::LabelFilterArgs;
use crate::runtime::{CliRuntimeCtx, send};

#[derive(Args, Debug)]
pub struct BlockedArgs {
    #[command(flatten)]
    pub label_filter: LabelFilterArgs,
}

pub fn handle(ctx: &CliRuntimeCtx, args: BlockedArgs) -> CommandResult<()> {
    let req = Request::Blocked {
        ctx: ctx.read_ctx(),
        payload: BlockedPayload {
            label_filter: args.label_filter.resolve(ctx),
        },
    };
    let ok = send(&req)?;
    print_ok(&ok, ctx.json)
}

pub fn render_blocked(blocked: &[beads_api::BlockedIssue]) -> String {
    if blocked.is_empty() {
        return "\n✨ No blocked issues\n".into();
    }

    let mut out = format!("\n🚫 Blocked issues ({}):\n\n", blocked.len());
    for blocked_issue in blocked {
        out.push_str(&format!(
            "[P{}] {}: {}\n",
            blocked_issue.issue.priority,
            fmt_issue_ref(&blocked_issue.issue.namespace, &blocked_issue.issue.id),
            blocked_issue.issue.title
        ));
        out.push_str(&format!(
            "  Blocked by {} open dependencies: {:?}\n\n",
            blocked_issue.blocked_by_count, blocked_issue.blocked_by
        ));
    }
    out.trim_end().into()
}
