use std::{env::var, sync::Once};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::{fmt, EnvFilter, FmtSubscriber};

use super::error::TracingSetupError;

static TRACING_INIT: Once = Once::new();

pub fn setup_tracing() -> Result<(), TracingSetupError> {
    let mut init_result: Result<(), TracingSetupError> = Ok(());

    // ensures that the subscriber is only initialized once for all threads
    TRACING_INIT.call_once(|| {
        let filter = EnvFilter::new(var("RUST_LOG").unwrap_or_else(|_| String::from("info")));

        let subscriber = FmtSubscriber::builder()
            .with_env_filter(filter)
            .with_timer(UtcTime::rfc_3339())
            .with_ansi(false)
            .fmt_fields(fmt::format::DefaultFields::new())
            .event_format(
                fmt::format()
                    .compact()
                    .with_line_number(true)
                    .with_thread_ids(true),
            )
            .finish();

        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            init_result = Err(e.into());
        }
    });
    init_result
}
