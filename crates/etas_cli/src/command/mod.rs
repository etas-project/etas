#[cfg(feature = "cli-check")]
pub mod check;
#[cfg(feature = "cli-dump-air")]
pub mod dump_air;
#[cfg(feature = "cli-dump-ast")]
pub mod dump_ast;
#[cfg(feature = "cli-dump-hir")]
pub mod dump_hir;
#[cfg(feature = "cli-effects")]
pub mod effects;
#[cfg(feature = "component-frontend")]
mod frontend;
#[cfg(feature = "cli-graph")]
pub mod graph;
#[cfg(any(feature = "cli-run", feature = "cli-resume"))]
pub mod interpreter;
#[cfg(feature = "cli-lsp")]
pub mod lsp;
#[cfg(feature = "cli-pkg")]
pub mod pkg;
#[cfg(feature = "cli-policy")]
pub mod policy;
#[cfg(feature = "cli-repl")]
pub mod repl;
#[cfg(feature = "cli-replay")]
pub mod replay;
#[cfg(feature = "cli-resume")]
pub mod resume;
#[cfg(feature = "cli-run")]
pub mod run;
#[cfg(feature = "cli-watch")]
pub mod watch;

use std::io::Write;

use etas_utils::{
    ProfileHandle, ProfileTreeRenderOptions, render_profile_tree_with_options, write_profile_report,
};

use crate::{
    args::{CliArgs, Command},
    config,
    error::CliError,
    exit::CliExit,
    workspace,
};

#[cfg(any(
    feature = "cli-dump-ast",
    feature = "cli-dump-hir",
    feature = "cli-dump-air"
))]
use crate::args::dump::{DumpArgs, DumpCommand};

pub fn dispatch(
    args: CliArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    let workspace = workspace::resolve_workspace(&args.global)?;
    if let Some(config) = config::load_config(&args.global, &workspace)? {
        tracing::debug!(path = %config.path.display(), bytes = config.contents.len(), "loaded etas config");
    }
    let command_name = command_name(&args.command);
    let profile = if args.global.profile_out.is_some() || args.global.profile_tree {
        ProfileHandle::enabled(command_name)
    } else {
        ProfileHandle::disabled()
    };
    let total_span = profile.span(format!("cli.{command_name}.total"), "cli");

    let result = match args.command {
        #[cfg(feature = "cli-check")]
        Command::Check(command) => check::run(&args.global, command, &profile, stdout, stderr),
        #[cfg(any(
            feature = "cli-dump-ast",
            feature = "cli-dump-hir",
            feature = "cli-dump-air"
        ))]
        Command::Dump(command) => dispatch_dump(&args.global, command, stdout, stderr),
        #[cfg(feature = "cli-graph")]
        Command::Graph(command) => graph::run(&args.global, command, stdout, stderr),
        #[cfg(feature = "cli-effects")]
        Command::Effects(command) => effects::run(&args.global, command, &profile, stdout, stderr),
        #[cfg(feature = "cli-policy")]
        Command::Policy(command) => policy::run(&args.global, command, stdout, stderr),
        #[cfg(feature = "cli-pkg")]
        Command::Pkg(command) => pkg::run(&args.global, command, &profile, stdout, stderr),
        #[cfg(feature = "cli-run")]
        Command::Run(command) => run::run(&args.global, command, &profile, stdout, stderr),
        #[cfg(feature = "cli-replay")]
        Command::Replay(command) => replay::run(&args.global, command, stdout, stderr),
        #[cfg(feature = "cli-resume")]
        Command::Resume(command) => resume::run(&args.global, command, stdout, stderr),
        #[cfg(feature = "cli-watch")]
        Command::Watch(command) => watch::run(&args.global, command, stdout, stderr),
        #[cfg(feature = "cli-repl")]
        Command::Repl(command) => repl::run(&args.global, command, stdout, stderr),
        #[cfg(feature = "cli-lsp")]
        Command::Lsp(command) => lsp::run(&args.global, command, stdout, stderr),
    };

    let status = match &result {
        Ok(CliExit::Success) => "ok",
        Ok(_) => "error",
        Err(_) => "error",
    };
    if status == "ok" {
        total_span.finish_ok();
    } else {
        total_span.finish_error();
    }
    let report = profile.finish_report(status);
    if let Some(path) = &args.global.profile_out
        && let Some(report) = &report
    {
        write_profile_report(path, report).map_err(|source| CliError::Io {
            path: path.clone(),
            source,
        })?;
    }
    if args.global.profile_tree
        && let Some(report) = &report
    {
        write!(
            stderr,
            "{}",
            render_profile_tree_with_options(
                report,
                ProfileTreeRenderOptions {
                    include_detail: args.global.profile_detail,
                    include_pass_timing: args.global.profile_pass_timing,
                },
            )
        )
        .map_err(|source| CliError::Io {
            path: "<stderr>".into(),
            source,
        })?;
    }
    result
}

fn command_name(command: &Command) -> &'static str {
    match command {
        #[cfg(feature = "cli-check")]
        Command::Check(_) => "check",
        #[cfg(any(
            feature = "cli-dump-ast",
            feature = "cli-dump-hir",
            feature = "cli-dump-air"
        ))]
        Command::Dump(_) => "dump",
        #[cfg(feature = "cli-graph")]
        Command::Graph(_) => "graph",
        #[cfg(feature = "cli-effects")]
        Command::Effects(_) => "effects",
        #[cfg(feature = "cli-policy")]
        Command::Policy(_) => "policy",
        #[cfg(feature = "cli-pkg")]
        Command::Pkg(_) => "pkg",
        #[cfg(feature = "cli-run")]
        Command::Run(_) => "run",
        #[cfg(feature = "cli-replay")]
        Command::Replay(_) => "replay",
        #[cfg(feature = "cli-resume")]
        Command::Resume(_) => "resume",
        #[cfg(feature = "cli-watch")]
        Command::Watch(_) => "watch",
        #[cfg(feature = "cli-repl")]
        Command::Repl(_) => "repl",
        #[cfg(feature = "cli-lsp")]
        Command::Lsp(_) => "lsp",
    }
}

#[cfg(any(
    feature = "cli-dump-ast",
    feature = "cli-dump-hir",
    feature = "cli-dump-air"
))]
fn dispatch_dump(
    global: &crate::args::global::GlobalOptions,
    args: DumpArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<CliExit, CliError> {
    match args.command {
        #[cfg(feature = "cli-dump-ast")]
        DumpCommand::Ast(command) => dump_ast::run(global, command, stdout, stderr),
        #[cfg(feature = "cli-dump-hir")]
        DumpCommand::Hir(command) => dump_hir::run(global, command, stdout, stderr),
        #[cfg(feature = "cli-dump-air")]
        DumpCommand::Air(command) => dump_air::run(global, command, stdout, stderr),
    }
}

#[cfg(any(
    feature = "cli-dump-air",
    feature = "cli-graph",
    feature = "cli-policy",
    feature = "cli-repl",
    feature = "cli-lsp",
))]
fn unsupported(command: &'static str, reason: &'static str) -> CliError {
    CliError::Unsupported { command, reason }
}
