use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
};

/// Sets up console logging with detailed output for local development
pub fn setup_console_tracing() {
    // Create an EnvFilter that captures all log levels by default
    // You can override this with the RUST_LOG environment variable
    let env_filter = EnvFilter::new("info");

    // Create a file appender that outputs JSON logs
    // let file_appender = tracing_appender::rolling::daily("../logs/", "application.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Setup the subscriber with pretty console output
    let subscriber = fmt::Subscriber::builder()
        // Log all spans and events (including entering/exiting spans)
        .with_span_events(FmtSpan::FULL)
        // Show source code location (file + line number)
        .with_file(true)
        .with_line_number(true)
        // Show target module path
        .with_target(true)
        // Use ANSI colors in output
        .with_ansi(true)
        // Display thread IDs
        .with_thread_ids(true)
        // Set the filter based on the environment
        .with_env_filter(env_filter)
        // .json()
        // .with_writer(non_blocking)
        // Build the subscriber
        .finish();

    // Set the subscriber as the default
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up the global tracing subscriber");

    // Log the fact that tracing has been initialized
    tracing::info!("Tracing initialized with console output");
}
