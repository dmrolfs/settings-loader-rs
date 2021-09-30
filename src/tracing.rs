use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

lazy_static::lazy_static! {
    pub static ref TEST_TRACING: () = {
        let filter = if std::env::var("TEST_LOG").is_ok() { "debug" } else { "" };
        let subscriber = get_subscriber("test", filter, std::io::stdout);
        init_subscriber(subscriber);
    };
}

pub fn get_subscriber(
    name: impl Into<String>, env_filter: impl AsRef<str>, sink: impl MakeWriter + Send + Sync + 'static,
) -> impl Subscriber + Sync + Send {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name.into(), sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
