use clap::Args;

#[derive(Args, Clone, Debug, Default)]
pub struct OptimizationArgs {
    #[arg(
        short = 'O',
        long = "opt-level",
        value_name = "LEVEL",
        value_parser = clap::value_parser!(u8).range(0..=3),
        action = clap::ArgAction::Append,
        help = "Compiler optimization level 0..3 (default: 2)"
    )]
    levels: Vec<u8>,
}

impl OptimizationArgs {
    pub fn level(&self) -> Result<u8, String> {
        let Some((&first, rest)) = self.levels.split_first() else {
            return Ok(2);
        };
        if rest.iter().any(|level| *level != first) {
            return Err("conflicting optimization levels were supplied".to_owned());
        }
        Ok(first)
    }
}

#[cfg(all(
    test,
    feature = "cli-check",
    feature = "cli-run",
    feature = "cli-dump-hir"
))]
mod tests {
    use clap::Parser;

    use crate::args::{
        CliArgs, Command,
        dump::{DumpCommand, DumpHirArgs},
    };

    fn check_level(args: &[&str]) -> Result<u8, String> {
        let parsed = CliArgs::try_parse_from(args).map_err(|error| error.to_string())?;
        let Command::Check(command) = parsed.command else {
            panic!("expected check command")
        };
        command.optimization.level()
    }

    #[test]
    fn parses_short_long_and_default_levels() {
        assert_eq!(check_level(&["etas", "check"]).unwrap(), 2);
        for level in 0..=3 {
            let short = format!("-O{level}");
            assert_eq!(check_level(&["etas", "check", &short]).unwrap(), level);
            let long = level.to_string();
            assert_eq!(
                check_level(&["etas", "check", "--opt-level", &long]).unwrap(),
                level
            );
        }
    }

    #[test]
    fn rejects_missing_out_of_range_and_conflicting_levels() {
        assert!(CliArgs::try_parse_from(["etas", "check", "-O"]).is_err());
        assert!(CliArgs::try_parse_from(["etas", "check", "-O4"]).is_err());
        assert!(check_level(&["etas", "check", "-O1", "--opt-level", "2"]).is_err());
        assert_eq!(
            check_level(&["etas", "check", "-O1", "--opt-level", "1"]).unwrap(),
            1
        );
    }

    #[test]
    fn run_and_hir_dump_share_the_argument_model() {
        let run = CliArgs::try_parse_from(["etas", "run", "-O3"]).unwrap();
        let Command::Run(run) = run.command else {
            panic!("expected run command")
        };
        assert_eq!(run.optimization.level().unwrap(), 3);

        let dump = CliArgs::try_parse_from(["etas", "dump", "hir", "-O0", "input.es"]).unwrap();
        let Command::Dump(dump) = dump.command else {
            panic!("expected dump command")
        };
        let DumpCommand::Hir(DumpHirArgs { optimization, .. }) = dump.command else {
            panic!("expected HIR dump command")
        };
        assert_eq!(optimization.level().unwrap(), 0);
    }
}
