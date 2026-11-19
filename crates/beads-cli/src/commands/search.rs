use beads_surface::Filters;
use beads_surface::ipc::{ListPayload, Request};
use clap::Args;

use super::{CommandResult, print_ok};
use crate::filters::LabelFilterArgs;
use crate::runtime::{CliRuntimeCtx, send};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query (multiple words allowed).
    #[arg(num_args = 1..)]
    pub query: Vec<String>,

    /// Limit results.
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    #[command(flatten)]
    pub label_filter: LabelFilterArgs,
}

pub fn handle(ctx: &CliRuntimeCtx, args: SearchArgs) -> CommandResult<()> {
    let filters = Filters {
        search: Some(args.query.join(" ")),
        limit: args.limit,
        labels: args.label_filter.labels(),
        labels_any: args.label_filter.labels_any(),
        ..Default::default()
    };
    let req = Request::List {
        ctx: ctx.read_ctx(),
        payload: ListPayload { filters },
    };
    let ok = send(&req)?;
    print_ok(&ok, ctx.json)
}
