use axiom_rs::Client;
use color_eyre::Result;
use serde_json::json;
use tracing_subscriber::{layer::Layered, EnvFilter, Registry};
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

pub type CompactRegistry = Layered<
    EnvFilter,
    Layered<
        tracing_subscriber::fmt::Layer<
            Registry,
            tracing_subscriber::fmt::format::DefaultFields,
            tracing_subscriber::fmt::format::Format<tracing_subscriber::fmt::format::Compact>,
        >,
        Registry,
    >,
>;

fn registry() -> CompactRegistry {
    Registry::default()
        .with(tracing_subscriber::fmt::layer().compact())
        .with(EnvFilter::from_default_env())
}

pub fn init() -> Result<()> {
    registry().try_init()?;

    Ok(())
}

#[derive(Debug, strum::Display)]
pub enum Event {
    Installation,
    NoAwsInstalled,
}

#[tracing::instrument(skip(metadata))]
pub async fn track_event<T>(event: Event, metadata: Option<T>) -> Result<()>
where
    T: serde::Serialize,
{
    let client = Client::builder().with_token(env!("AXIOM_TOKEN")).build()?;

    client
        .ingest(
            env!("AXIOM_DATASET"),
            [json!({
                "event_name": event.to_string(),
                "metadata": metadata
            })],
        )
        .await?;

    Ok(())
}
