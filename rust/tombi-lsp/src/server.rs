mod backend;
mod handler;

use backend::Backend;
use tower_lsp::LspService;
use tower_lsp::Server;

use crate::version::version;

/// Run TOML Language Server
#[derive(clap::Args, Debug)]
pub struct Args {}

pub async fn run(_args: impl Into<Args>) -> Result<(), anyhow::Error> {
    tracing::info!("Tombi LSP Server Version \"{}\" will start.", version());

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    tracing::info!("Tombi LSP Server did shut down.");

    Ok(())
}
