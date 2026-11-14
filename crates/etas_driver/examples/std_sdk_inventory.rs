//! Export the selected SDK query view for the existing per-module migration audit.
use std::{error::Error, io, path::PathBuf};

use etas_driver::std_library::provider::StdProvider;
use etas_std::migration::StdMigrationBaseline;

fn main() -> Result<(), Box<dyn Error>> {
    let sysroot = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("usage: std_sdk_inventory <sysroot>")?,
    );
    let StdProvider::Artifact(provider) = StdProvider::artifact(Some(&sysroot))? else {
        unreachable!()
    };
    serde_json::to_writer_pretty(
        io::stdout().lock(),
        &StdMigrationBaseline::from_registry(&provider.registry),
    )?;
    Ok(())
}
