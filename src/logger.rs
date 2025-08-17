use chrono_tz::Asia::Shanghai;
use owo_colors::OwoColorize;
use std::fmt;
use tracing::Subscriber;
use tracing_subscriber::{
    Layer,
    filter::LevelFilter,
    fmt::{FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
};

struct Formatter;

impl<S, N> FormatEvent<S, N> for Formatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        let prefix = "[image-spider]".magenta().to_string();
        write!(writer, "{} ", prefix)?;

        let local_time = chrono::Local::now();
        let shanghai_time = local_time.with_timezone(&Shanghai);
        let formatted_time = shanghai_time.format("%H:%M:%S%.3f");
        write!(writer, "[{}] ", formatted_time)?;

        let logger_level = event.metadata().level();
        let colored_level = match *logger_level {
            tracing::Level::ERROR => logger_level.red().to_string(),
            tracing::Level::WARN => logger_level.yellow().to_string(),
            tracing::Level::INFO => logger_level.green().to_string(),
            tracing::Level::DEBUG => logger_level.blue().to_string(),
            tracing::Level::TRACE => logger_level.magenta().to_string(),
        };
        write!(writer, "[{: <17}] ", colored_level)?;

        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn log_init() {
    let logger_level = LevelFilter::INFO;

    let console_subscriber = tracing_subscriber::fmt::layer()
        .event_format(Formatter {})
        .with_filter(logger_level);

    let subscriber = tracing_subscriber::registry().with(console_subscriber);

    if tracing::subscriber::set_global_default(subscriber).is_err()
        || tracing_log::LogTracer::init().is_err()
    {
        return;
    }
}
