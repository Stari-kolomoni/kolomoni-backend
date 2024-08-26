use miette::{Context, IntoDiagnostic, Result};

use crate::cli::DownCommandArguments;

pub fn cli_down(arguments: DownCommandArguments) -> Result<()> {
    let async_runtime = tokio::runtime::Runtime::new()
        .into_diagnostic()
        .wrap_err("failed to initialize tokio async runtime")?;

    async_runtime
        .block_on(cli_down_inner(arguments))
        .wrap_err("failed to run root async task to completion")
}


async fn cli_down_inner(arguments: DownCommandArguments) -> Result<()> {
    todo!();
}
