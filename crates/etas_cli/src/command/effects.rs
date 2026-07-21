use std::{io::Write, path::PathBuf};

use etas_core::{Diagnostic, Severity, SourceFile};
use etas_driver::SourceDependencyMode;
use etas_effects::{
    ActionRef, Effect, EffectRegistry, EffectRow, EffectSummary, EffectTagId, InterpreterSupport,
};
use etas_frontend::{EntryPolicy, ProjectOutput};
use etas_hir::{HirItem, HirItemId, SymbolId};
use etas_utils::ProfileHandle;
use serde::Serialize;
use serde_json::json;

use crate::{
    args::{EffectsArgs, global::GlobalOptions},
    error::CliError,
    exit::CliExit,
    output::{diagnostic, text},
};

#[derive(Serialize)]
struct EffectsReport {
    command: &'static str,
    paths: Vec<PathBuf>,
    flow: Option<String>,
    items: Vec<EffectItemReport>,
}

#[derive(Serialize)]
struct EffectItemReport {
    id: u32,
    kind: &'static str,
    name: String,
    escaping_effects: String,
    requested_actions: String,
    default_actions: String,
    residual_checks: Vec<String>,
    trace_spec_obligations: Vec<String>,
    requirements: Vec<String>,
    determinism: String,
    support: String,
    summary: serde_json::Value,
}

pub fn run(
    global: &GlobalOptions,
    args: EffectsArgs,
    profile: &ProfileHandle,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    let loaded = super::frontend::load_project(
        global,
        profile,
        vec![args.file],
        false,
        EntryPolicy::CompileOnly,
        args.flow.clone(),
        SourceDependencyMode::MetadataOnly,
    )?;
    let sources = loaded.sources;
    let output = super::frontend::check_project_once(global, profile, loaded.input)?;
    write_diagnostics(stderr, &output.diagnostics, &sources)?;

    let report = effects_report(&output, &sources, args.flow.as_deref())?;
    render_report(global, stdout, &report, &output)?;

    if has_blocking_effect_errors(&output.diagnostics) || output.effects.is_none() {
        Ok(CliExit::Diagnostics)
    } else {
        Ok(CliExit::Success)
    }
}

fn effects_report(
    output: &ProjectOutput,
    sources: &[(PathBuf, SourceFile)],
    flow: Option<&str>,
) -> Result<EffectsReport, CliError> {
    let Some(hir) = output.hir.as_ref() else {
        return Ok(EffectsReport {
            command: "effects",
            paths: source_paths(sources),
            flow: flow.map(str::to_owned),
            items: Vec::new(),
        });
    };
    let Some(effects) = output.effects.as_ref() else {
        return Ok(EffectsReport {
            command: "effects",
            paths: source_paths(sources),
            flow: flow.map(str::to_owned),
            items: Vec::new(),
        });
    };
    let Some(artifacts) = output.effect_pipeline_artifacts.as_ref() else {
        return Err(CliError::InvalidUsage(
            "effect report is missing effect registry artifacts".to_owned(),
        ));
    };

    let mut items = effects
        .facts
        .item_effects
        .iter()
        .filter_map(|(id, summary)| {
            let item = hir.hir.items.get(*id)?;
            let symbol = item_symbol(item)?;
            let symbol = hir.hir.symbols.get(symbol)?;
            let kind = item_kind(item);
            if flow.is_some_and(|flow| kind != "flow" || symbol.name != flow) {
                return None;
            }
            Some(effect_item_report(
                *id,
                kind,
                symbol.name.clone(),
                summary,
                &artifacts.registry,
                output.types.as_ref().map(|types| &types.store),
            ))
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    items.sort_by_key(|item| item.id);

    Ok(EffectsReport {
        command: "effects",
        paths: source_paths(sources),
        flow: flow.map(str::to_owned),
        items,
    })
}

fn effect_item_report(
    id: HirItemId,
    kind: &'static str,
    name: String,
    summary: &EffectSummary,
    registry: &EffectRegistry,
    types: Option<&etas_types::TypeStore>,
) -> Result<EffectItemReport, CliError> {
    Ok(EffectItemReport {
        id: id.0,
        kind,
        name,
        escaping_effects: effect_row_text(&summary.escaping_effects, registry, types),
        requested_actions: effect_row_text(&summary.requested_actions, registry, types),
        default_actions: effect_row_text(&summary.default_actions, registry, types),
        residual_checks: summary.residual_checks.checks.iter().cloned().collect(),
        trace_spec_obligations: summary
            .trace_spec_obligations
            .iter()
            .map(requirement_text)
            .collect(),
        requirements: summary.requirements.iter().map(requirement_text).collect(),
        determinism: format!("{:?}", summary.determinism),
        support: support_text(&summary.support),
        summary: serde_json::to_value(summary).map_err(|error| {
            CliError::InvalidUsage(format!("effect report is not JSON: {error}"))
        })?,
    })
}

fn requirement_text(requirement: &etas_effects::RequirementFact) -> String {
    match requirement {
        etas_effects::RequirementFact::TraceSpec(policy) => policy.label(),
        other => format!("{other:?}"),
    }
}

fn render_report(
    global: &GlobalOptions,
    stdout: &mut dyn Write,
    report: &EffectsReport,
    output: &ProjectOutput,
) -> Result<(), CliError> {
    match global.format {
        crate::args::global::OutputFormat::Human | crate::args::global::OutputFormat::Text => {
            writeln!(
                stdout,
                "effects {}",
                text::plural(report.items.len(), "item", "items")
            )
            .map_err(stdout_error)?;
            for item in &report.items {
                writeln!(stdout, "{} {}", item.kind, item.name).map_err(stdout_error)?;
                writeln!(stdout, "  escaping effects: {}", item.escaping_effects)
                    .map_err(stdout_error)?;
                writeln!(stdout, "  requested actions: {}", item.requested_actions)
                    .map_err(stdout_error)?;
                writeln!(stdout, "  default actions: {}", item.default_actions)
                    .map_err(stdout_error)?;
                if !item.residual_checks.is_empty() {
                    writeln!(
                        stdout,
                        "  residual checks: {}",
                        item.residual_checks.join(", ")
                    )
                    .map_err(stdout_error)?;
                }
                if !item.trace_spec_obligations.is_empty() {
                    writeln!(
                        stdout,
                        "  trace spec obligations: {}",
                        item.trace_spec_obligations.join(", ")
                    )
                    .map_err(stdout_error)?;
                }
                writeln!(stdout, "  determinism: {}", item.determinism).map_err(stdout_error)?;
                writeln!(stdout, "  support: {}", item.support).map_err(stdout_error)?;
                if !item.requirements.is_empty() {
                    writeln!(stdout, "  requirements: {}", item.requirements.join(", "))
                        .map_err(stdout_error)?;
                }
            }
        }
        crate::args::global::OutputFormat::Json => {
            writeln!(
                stdout,
                "{}",
                json!({
                    "command": report.command,
                    "paths": report.paths,
                    "flow": report.flow,
                    "items": report.items,
                    "diagnostics": output.diagnostics,
                })
            )
            .map_err(stdout_error)?;
        }
        crate::args::global::OutputFormat::Jsonl => {
            for item in &report.items {
                writeln!(
                    stdout,
                    "{}",
                    json!({
                        "type": "effect_item",
                        "item": item,
                    })
                )
                .map_err(stdout_error)?;
            }
            writeln!(
                stdout,
                "{}",
                json!({
                    "type": "summary",
                    "command": report.command,
                    "paths": report.paths,
                    "flow": report.flow,
                    "items": report.items.len(),
                })
            )
            .map_err(stdout_error)?;
        }
        other => {
            return Err(CliError::InvalidUsage(format!(
                "`--format {other:?}` is not valid for `etas effects`"
            )));
        }
    }
    Ok(())
}

fn write_diagnostics(
    stderr: &mut dyn Write,
    diagnostics: &[Diagnostic],
    sources: &[(PathBuf, SourceFile)],
) -> Result<(), CliError> {
    for (_, source) in sources {
        let source_diagnostics = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.primary.span.source == source.id)
            .cloned()
            .collect::<Vec<_>>();
        if source_diagnostics.is_empty() {
            continue;
        }
        write!(
            stderr,
            "{}",
            diagnostic::render_human(&source_diagnostics, source)
        )
        .map_err(|source| CliError::Io {
            path: "<stderr>".into(),
            source,
        })?;
    }
    Ok(())
}

fn has_blocking_effect_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

fn source_paths(sources: &[(PathBuf, SourceFile)]) -> Vec<PathBuf> {
    sources.iter().map(|(path, _)| path.clone()).collect()
}

fn item_symbol(item: &HirItem) -> Option<SymbolId> {
    Some(match item {
        HirItem::Flow(item) => item.symbol,
        HirItem::Agent(item) => item.symbol,
        HirItem::Tool(item) => item.symbol,
        HirItem::Effect(item) => item.symbol,
        _ => return None,
    })
}

fn item_kind(item: &HirItem) -> &'static str {
    match item {
        HirItem::Flow(_) => "flow",
        HirItem::Agent(_) => "agent",
        HirItem::Tool(_) => "tool",
        HirItem::Effect(_) => "effect",
        _ => "item",
    }
}

fn effect_row_text(
    row: &EffectRow,
    registry: &EffectRegistry,
    types: Option<&etas_types::TypeStore>,
) -> String {
    let mut effects = row
        .effects
        .iter()
        .map(|effect| effect_text(effect, registry, types))
        .collect::<Vec<_>>();
    if let Some(open) = row.open {
        effects.push(format!("'{}", open.0));
    }
    if effects.is_empty() {
        "[]".to_owned()
    } else {
        format!("[{}]", effects.join(", "))
    }
}

fn effect_text(
    effect: &Effect,
    registry: &EffectRegistry,
    types: Option<&etas_types::TypeStore>,
) -> String {
    match effect {
        Effect::Tag(tag) => effect_tag_name(*tag, registry),
        Effect::Action(action) => effect_action_name(action, registry),
        Effect::AppliedAction(action) => {
            let args = action
                .args
                .iter()
                .map(|arg| effect_arg_text(arg, types))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}[{args}]", effect_action_name(&action.action, registry))
        }
        Effect::Applied { tag, args } => {
            let args = args
                .iter()
                .map(|arg| type_text(*arg, types))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}[{args}]", effect_tag_name(*tag, registry))
        }
        Effect::Var(var) => format!("'{}", var.0),
        Effect::Error(error) => format!("Error<{}>", type_text(*error, types)),
    }
}

fn effect_arg_text(
    arg: &etas_types::EffectArgRef,
    types: Option<&etas_types::TypeStore>,
) -> String {
    match arg {
        etas_types::EffectArgRef::Type(ty) => type_text(*ty, types),
        etas_types::EffectArgRef::Path(segments) => segments.join("."),
        etas_types::EffectArgRef::Wildcard => "_".to_owned(),
        etas_types::EffectArgRef::String(value) => format!("{value:?}"),
        etas_types::EffectArgRef::Int(value) => value.to_string(),
    }
}

fn type_text(ty: etas_types::TypeId, types: Option<&etas_types::TypeStore>) -> String {
    let Some(types) = types else {
        return format!("{ty:?}");
    };
    match types.get(ty) {
        Some(etas_types::Type::Named(named)) => named.name.clone(),
        Some(etas_types::Type::Nominal(nominal)) => nominal.name.clone(),
        Some(etas_types::Type::Enum(named)) => named.name.clone(),
        Some(etas_types::Type::MemoryPlace(place)) => place.segments.join("."),
        Some(etas_types::Type::Primitive(primitive)) => primitive.source_name().to_owned(),
        Some(other) => format!("{other:?}"),
        None => format!("{ty:?}"),
    }
}

fn effect_tag_name(tag: EffectTagId, registry: &EffectRegistry) -> String {
    registry
        .tag_name(tag)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("UnknownTag({})", tag.0))
}

fn effect_action_name(action: &ActionRef, registry: &EffectRegistry) -> String {
    let owner = effect_tag_name(action.tag, registry);
    let action_name = registry
        .action_name(action.tag, action.action)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("UnknownAction({})", action.action.0));
    format!("{owner}.{action_name}")
}

fn support_text(support: &InterpreterSupport) -> String {
    match support {
        InterpreterSupport::LocalOnly => "local".to_owned(),
        InterpreterSupport::RequiresHost(requirements) => {
            let requirements = requirements
                .kinds
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("requires-host[{requirements}]")
        }
        InterpreterSupport::RequiresInterpreterOrchestration(requirements) => {
            let host = requirements
                .host
                .kinds
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            let features = requirements
                .features
                .kinds
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("requires-orchestration[host={host}; features={features}]")
        }
        InterpreterSupport::Rejected(reason) => format!("rejected[{reason:?}]"),
    }
}

fn stdout_error(source: std::io::Error) -> CliError {
    CliError::Io {
        path: "<stdout>".into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use etas_effects::{CoreEffect, Effect, EffectRegistry};
    use etas_types::{NamedTypeRef, Type, TypeInterner};

    #[test]
    fn effect_text_prints_named_error_type_instead_of_type_id() {
        let registry = EffectRegistry::with_standard_effects();
        let mut interner = TypeInterner::new();
        let io_error = interner.intern(Type::Named(NamedTypeRef {
            name: "IOError".to_owned(),
        }));
        let index_error = interner.intern(Type::Named(NamedTypeRef {
            name: "IndexError".to_owned(),
        }));
        let store = interner.into_store();

        assert_eq!(
            super::effect_text(&Effect::Error(io_error), &registry, Some(&store)),
            "Error<IOError>"
        );
        assert_eq!(
            super::effect_text(&Effect::Error(index_error), &registry, Some(&store)),
            "Error<IndexError>"
        );
    }

    #[test]
    fn effect_text_uses_registry_for_standard_action_names() {
        let registry = EffectRegistry::with_standard_effects();
        let action = registry
            .core_action(CoreEffect::Agentic, etas_effects::AGENTIC_INFER_ACTION)
            .expect("Agentic.infer action exists");
        assert_eq!(
            super::effect_text(&Effect::Action(action), &registry, None),
            "Agentic.infer"
        );
    }
}
