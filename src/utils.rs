// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn setup_logging(debug: bool) {
    let log_level = if debug { Level::DEBUG } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(log_level)
        // trace to stderr
        .with_writer(std::io::stderr)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
