//! # How to build
//! ```bash
//! cargo build --example lsp_server --package datex-core --features "lsp_tokio lsp_example"
//! ```

use datex_core::{lsp::create_lsp, runtime::Runtime};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a stub runtime, LSP doesn't need network/network
    // The runtime provides the compiler context
    let runtime = Runtime::stub();

    // Start the LSP server
    // This runs forever, listening for LSP requests on stdin
    // and responding on stdout
    create_lsp(runtime, tokio::io::stdin(), tokio::io::stdout()).await;

    Ok(())
}
