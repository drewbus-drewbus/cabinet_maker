use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "cm-server", version, about = "Cabinet Maker web server")]
struct Args {
    /// Host address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value = "3001")]
    port: u16,

    /// Path to templates directory
    #[arg(long, default_value = "templates")]
    templates: PathBuf,

    /// Path to static files directory (SvelteKit build output)
    #[arg(long)]
    static_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    tracing::info!(
        "Starting cm-server on {}:{} (templates: {})",
        args.host, args.port, args.templates.display()
    );

    let app = cm_server::build_app(args.templates, args.static_dir);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port))
        .await
        .expect("Failed to bind address");

    tracing::info!("Listening on http://{}:{}", args.host, args.port);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
