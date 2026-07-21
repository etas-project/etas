pub mod args;
pub mod command;
pub mod config;
pub mod error;
pub mod exit;
pub mod output;
pub mod workspace;

#[cfg(not(any(
    feature = "cli-check",
    feature = "cli-dump-ast",
    feature = "cli-dump-hir",
    feature = "cli-dump-air",
    feature = "cli-graph",
    feature = "cli-effects",
    feature = "cli-policy",
    feature = "cli-pkg",
    feature = "cli-run",
    feature = "cli-replay",
    feature = "cli-resume",
    feature = "cli-watch",
    feature = "cli-repl",
    feature = "cli-lsp",
)))]
compile_error!("etas_cli requires at least one cli-* feature");

use std::{ffi::OsString, io::Write, path::Path};

use args::CliArgs;
use clap::Parser;

pub fn run() -> i32 {
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    run_with(std::env::args_os(), &mut stdout, &mut stderr)
}

pub fn run_with<I, T>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = normalize_default_run_args(args);
    let args = match CliArgs::try_parse_from(args) {
        Ok(args) => args,
        Err(error) => {
            let code = if error.use_stderr() {
                exit::CliExit::Usage.code()
            } else {
                exit::CliExit::Success.code()
            };
            if error.use_stderr() {
                let _ = write!(stderr, "{error}");
            } else {
                let _ = write!(stdout, "{error}");
            }
            return code;
        }
    };

    match command::dispatch(args, stdout, stderr) {
        Ok(exit) => exit.code(),
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            error.exit().code()
        }
    }
}

fn normalize_default_run_args<I, T>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<OsString>>();
    normalize_optional_argv0(&mut args);
    if args.len() <= 1 {
        return args;
    }

    let mut index = 1;
    while index < args.len() {
        let Some(token) = args[index].to_str() else {
            insert_run(&mut args, index);
            return args;
        };
        if token == "--" {
            if index + 1 < args.len() {
                insert_run(&mut args, index + 1);
            }
            return args;
        }
        if is_help_or_version(token) {
            return args;
        }
        if is_known_subcommand(token) {
            return args;
        }
        if let Some(skip) = global_option_arity(token) {
            index += skip;
            continue;
        }
        if is_path_like_input(token) {
            insert_run(&mut args, index);
        }
        return args;
    }

    args
}

fn normalize_optional_argv0(args: &mut Vec<OsString>) {
    let Some(first) = args.first().and_then(|arg| arg.to_str()) else {
        return;
    };
    if is_known_subcommand(first) || is_help_or_version(first) || first.starts_with('-') {
        args.insert(0, OsString::from("etas"));
    }
}

fn insert_run(args: &mut Vec<OsString>, index: usize) {
    args.insert(index, OsString::from("run"));
}

fn is_help_or_version(token: &str) -> bool {
    matches!(token, "-h" | "--help" | "-V" | "--version")
}

fn is_known_subcommand(token: &str) -> bool {
    matches!(
        token,
        "check"
            | "dump"
            | "graph"
            | "effects"
            | "policy"
            | "pkg"
            | "run"
            | "replay"
            | "resume"
            | "watch"
            | "repl"
            | "lsp"
    )
}

fn is_path_like_input(token: &str) -> bool {
    token == "."
        || token == ".."
        || token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
        || token.contains('/')
        || token.ends_with(".es")
        || Path::new(token).exists()
}

fn global_option_arity(token: &str) -> Option<usize> {
    if matches!(token, "--no-config" | "--quiet") || is_verbose_flag(token) {
        return Some(1);
    }
    if token.starts_with("--workspace=")
        || token.starts_with("--config=")
        || token.starts_with("--format=")
        || token.starts_with("--color=")
        || token.starts_with("--log-level=")
        || token.starts_with("--cache=")
        || token.starts_with("--cache-root=")
    {
        return Some(1);
    }
    if matches!(
        token,
        "--workspace"
            | "--config"
            | "--format"
            | "--color"
            | "--log-level"
            | "--cache"
            | "--cache-root"
    ) {
        return Some(2);
    }
    None
}

fn is_verbose_flag(token: &str) -> bool {
    token.starts_with('-') && token.len() > 1 && token[1..].chars().all(|ch| ch == 'v')
}
