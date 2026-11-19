use beads_core::{ActorId, BeadType, CoreError, Priority};
use beads_surface::Filters;
use beads_surface::ipc::LabelFilter;

use crate::runtime::CliRuntimeCtx;
use clap::Args;
use clap::builder::NonEmptyStringValueParser;

use super::parsers::{parse_bead_type, parse_priority, parse_status, parse_time_ms};

pub type Result<T> = std::result::Result<T, FilterError>;

#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("validation failed for field {field}: {reason}")]
    Validation { field: String, reason: String },
}

pub fn validation_error(field: impl Into<String>, reason: impl Into<String>) -> FilterError {
    FilterError::Validation {
        field: field.into(),
        reason: reason.into(),
    }
}

/// Label filter flags shared by `list`, `count`, `ready`, `blocked`, `stale` and `search`.
#[derive(Args, Debug, Clone)]
pub struct LabelFilterArgs {
    // One value per occurrence: a variadic flag would swallow a following positional
    // (`bd search -l x timeout`) and could be given with no value at all.
    /// Filter by labels (AND: must have ALL). Repeat or comma-separated; `*` is a
    /// wildcard (`-l '*3d-sim*'`), otherwise labels match exactly.
    #[arg(
        short = 'l',
        long = "label",
        alias = "labels",
        value_delimiter = ',',
        value_parser = NonEmptyStringValueParser::new()
    )]
    pub labels: Vec<String>,

    /// Filter by labels (OR: must have AT LEAST ONE). Repeat or comma-separated; `*`
    /// is a wildcard, otherwise labels match exactly.
    #[arg(
        long = "label-any",
        value_delimiter = ',',
        value_parser = NonEmptyStringValueParser::new()
    )]
    pub labels_any: Vec<String>,
}

impl LabelFilterArgs {
    /// The effective label constraint: the flags when any is given, otherwise the
    /// runtime's default labels (`BD_LABEL` / `[defaults] labels`) as `--label`.
    ///
    /// Explicit flags replace the default rather than adding to it; each flag needs a
    /// value, so "given" and "non-empty" coincide. In human mode (not `--json`, not
    /// `--quiet`) a default in effect is noted on stderr, so filtered-out beads never
    /// look lost.
    pub fn resolve(&self, ctx: &CliRuntimeCtx) -> LabelFilter {
        if !self.labels.is_empty() || !self.labels_any.is_empty() {
            return self.explicit();
        }
        let Some(defaults) = &ctx.default_labels else {
            return LabelFilter::default();
        };
        if !ctx.json && !ctx.quiet {
            eprintln!(
                "note: default label filter -l {} (BD_LABEL or [defaults] labels); --no-default-labels shows all",
                defaults.join(",")
            );
        }
        LabelFilter {
            labels: Some(defaults.clone()),
            labels_any: None,
        }
    }

    /// The flags as given, without the default.
    fn explicit(&self) -> LabelFilter {
        LabelFilter {
            labels: non_empty(&self.labels),
            labels_any: non_empty(&self.labels_any),
        }
    }
}

fn non_empty(values: &[String]) -> Option<Vec<String>> {
    if values.is_empty() {
        None
    } else {
        Some(values.to_vec())
    }
}

#[derive(Args, Debug, Clone)]
pub struct CommonFilterArgs {
    /// Filter by status (todo, in_progress, human_review, rework, merging, done, cancelled, duplicate).
    #[arg(short = 's', long, value_parser = parse_status)]
    pub status: Option<beads_core::IssueStatus>,

    /// Filter by priority (0-4).
    #[arg(short = 'p', long, value_parser = parse_priority)]
    pub priority: Option<Priority>,

    /// Minimum priority (inclusive).
    #[arg(long = "priority-min", value_parser = parse_priority)]
    pub priority_min: Option<Priority>,

    /// Maximum priority (inclusive).
    #[arg(long = "priority-max", value_parser = parse_priority)]
    pub priority_max: Option<Priority>,

    /// Filter by type (bug, feature, task, epic, chore).
    #[arg(short = 't', long = "type", alias = "issue-type", value_parser = parse_bead_type)]
    pub bead_type: Option<BeadType>,

    /// Filter by assignee.
    #[arg(short = 'a', long)]
    pub assignee: Option<String>,

    #[command(flatten)]
    pub label_filter: LabelFilterArgs,

    /// Filter by title substring.
    #[arg(long = "title-contains")]
    pub title_contains: Option<String>,

    /// Filter by description substring.
    #[arg(long = "desc-contains")]
    pub desc_contains: Option<String>,

    /// Filter by notes substring.
    #[arg(long = "notes-contains")]
    pub notes_contains: Option<String>,

    /// Filter issues created after date (YYYY-MM-DD or RFC3339).
    #[arg(long = "created-after")]
    pub created_after: Option<String>,

    /// Filter issues created before date (YYYY-MM-DD or RFC3339).
    #[arg(long = "created-before")]
    pub created_before: Option<String>,

    /// Filter issues updated after date (YYYY-MM-DD or RFC3339).
    #[arg(long = "updated-after")]
    pub updated_after: Option<String>,

    /// Filter issues updated before date (YYYY-MM-DD or RFC3339).
    #[arg(long = "updated-before")]
    pub updated_before: Option<String>,

    /// Filter issues closed after date (YYYY-MM-DD or RFC3339).
    #[arg(long = "closed-after")]
    pub closed_after: Option<String>,

    /// Filter issues closed before date (YYYY-MM-DD or RFC3339).
    #[arg(long = "closed-before")]
    pub closed_before: Option<String>,

    /// Filter issues with empty description.
    #[arg(long = "empty-description")]
    pub empty_description: bool,

    /// Filter issues with no assignee.
    #[arg(long = "no-assignee")]
    pub no_assignee: bool,

    /// Filter issues with no labels.
    #[arg(long = "no-labels")]
    pub no_labels: bool,
}

pub fn apply_common_filters(
    args: &CommonFilterArgs,
    ctx: &CliRuntimeCtx,
    filters: &mut Filters,
) -> Result<()> {
    filters.status = args.status;
    filters.priority = args.priority;
    filters.priority_min = args.priority_min;
    filters.priority_max = args.priority_max;
    filters.bead_type = args.bead_type;
    filters.assignee = args.assignee.as_deref().map(ActorId::new).transpose()?;
    // `--no-labels` is a label constraint of its own, so it also replaces the default.
    let label_filter = if args.no_labels {
        args.label_filter.explicit()
    } else {
        args.label_filter.resolve(ctx)
    };
    filters.labels = label_filter.labels;
    filters.labels_any = label_filter.labels_any;
    filters.title_contains = args.title_contains.clone().filter(|s| !s.trim().is_empty());
    filters.desc_contains = args.desc_contains.clone().filter(|s| !s.trim().is_empty());
    filters.notes_contains = args.notes_contains.clone().filter(|s| !s.trim().is_empty());
    filters.created_after = parse_time_ms_opt(args.created_after.as_deref())?;
    filters.created_before = parse_time_ms_opt(args.created_before.as_deref())?;
    filters.updated_after = parse_time_ms_opt(args.updated_after.as_deref())?;
    filters.updated_before = parse_time_ms_opt(args.updated_before.as_deref())?;
    filters.closed_after = parse_time_ms_opt(args.closed_after.as_deref())?;
    filters.closed_before = parse_time_ms_opt(args.closed_before.as_deref())?;
    filters.empty_description = args.empty_description;
    filters.no_assignee = args.no_assignee;
    filters.no_labels = args.no_labels;
    Ok(())
}

fn parse_time_ms_opt(raw: Option<&str>) -> Result<Option<u64>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    parse_time_ms(trimmed)
        .map(Some)
        .map_err(|err| validation_error("date", err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Probe {
        #[command(flatten)]
        common: CommonFilterArgs,
    }

    fn ctx(default_labels: Option<&[&str]>) -> CliRuntimeCtx {
        CliRuntimeCtx {
            // JSON mode keeps the default-filter note off the test's stderr.
            json: true,
            default_labels: default_labels
                .map(|labels| labels.iter().map(|label| label.to_string()).collect()),
            ..Default::default()
        }
    }

    fn filters(args: &[&str], ctx: &CliRuntimeCtx) -> Filters {
        let probe = Probe::parse_from(std::iter::once("probe").chain(args.iter().copied()));
        let mut filters = Filters::default();
        apply_common_filters(&probe.common, ctx, &mut filters).expect("filters");
        filters
    }

    fn owned(values: &[&str]) -> Option<Vec<String>> {
        Some(values.iter().map(|value| value.to_string()).collect())
    }

    #[test]
    fn default_labels_apply_only_without_label_flags() {
        let scoped = ctx(Some(&["project:a", "p:*"]));

        let f = filters(&[], &scoped);
        assert_eq!(f.labels, owned(&["project:a", "p:*"]));
        assert_eq!(f.labels_any, None);

        // Every explicit label constraint replaces the default instead of adding to it.
        let f = filters(&["-l", "urgent"], &scoped);
        assert_eq!(f.labels, owned(&["urgent"]));
        assert_eq!(f.labels_any, None);

        let f = filters(&["--label-any", "x,y"], &scoped);
        assert_eq!(f.labels, None);
        assert_eq!(f.labels_any, owned(&["x", "y"]));

        let f = filters(&["--no-labels"], &scoped);
        assert_eq!((f.labels, f.labels_any, f.no_labels), (None, None, true));

        // Non-label filters keep the default.
        let f = filters(&["-s", "todo"], &scoped);
        assert_eq!(f.labels, owned(&["project:a", "p:*"]));
    }

    #[test]
    fn no_default_labels_means_no_label_constraint() {
        let f = filters(&[], &ctx(None));
        assert_eq!((f.labels, f.labels_any), (None, None));

        let f = filters(&["-l", "urgent"], &ctx(None));
        assert_eq!(f.labels, owned(&["urgent"]));
    }
}
