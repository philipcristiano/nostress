use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use nostr_sdk::prelude::*;
use rss::{ChannelBuilder, Item};
use serde::Deserialize;
use std::net::SocketAddr;
use std::ops::Sub;
use std::time::Duration;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "127.0.0.1:3000")]
    bind_addr: String,
    #[arg(short, long, num_args=0.. )]
    default_relay: Option<Vec<String>>,
    #[arg(short, long, value_enum, default_value = "INFO")]
    log_level: tracing::Level,
    #[arg(long, action)]
    log_json: bool,
}

#[derive(Clone, Debug)]
struct NostressConfig {
    default_relays: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    service_conventions::tracing::setup(args.log_level);

    let default_relays = args.default_relay.unwrap_or_default();

    let nostress_config = NostressConfig { default_relays };
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/users/:user_id/rss", get(user_rss))
        .with_state(nostress_config)
        .layer(service_conventions::tracing_http::trace_layer(Level::INFO))
        .route("/_health", get(health));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr: SocketAddr = args.bind_addr.parse().expect("Expected bind addr");
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn health() -> Response {
    "OK".into_response()
}

#[derive(Debug, Deserialize)]
struct TextNoteFilters {
    include_text_note_replies: Option<bool>,
}

#[tracing::instrument]
async fn get_user_profile(
    user_id: &String,
) -> anyhow::Result<nostr_sdk::nips::nip19::Nip19Profile> {
    Ok(nostr_sdk::nips::nip05::get_profile(&user_id, None).await?)
}

async fn user_rss(
    State(nc): State<NostressConfig>,
    Path(user_id): Path<String>,
    Query(text_note_filters): Query<TextNoteFilters>,
) -> Result<Response, AppError> {
    let profile = get_user_profile(&user_id).await?;
    let profile_key = Keys::from_public_key(profile.public_key);
    let client = Client::new(&profile_key);
    for r in profile.relays {
        client.add_relay(r).await?;
    }
    for default_relay in nc.default_relays {
        client.add_relay(default_relay).await?;
    }

    client.connect().await;

    let now = Timestamp::now();
    let since = now.sub(Duration::from_secs(86400));
    let subscription = Filter::new()
        .author(profile.public_key)
        .kind(Kind::TextNote)
        .since(since);

    let timeout = Duration::from_secs(10);
    let events = client
        .get_events_of(vec![subscription], Some(timeout))
        .await?;
    client.disconnect().await?;

    let title = vec![user_id, " - Nostr".to_string()].join("");

    let mut channel = ChannelBuilder::default()
        .title(&title)
        .link("http://example.com".to_string())
        .description(&title)
        .build();

    let mut items: Vec<Item> = Vec::new();

    let include_replies = text_note_filters.include_text_note_replies.unwrap_or(false);
    let filtered_events = if !include_replies {
        nostress::filter_out_replies(events)
    } else {
        events
    };

    for e in filtered_events {
        items.push(nostress::event_to_item(e));
    }

    channel.set_items(items);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/rss+xml")],
        channel.to_string(),
    )
        .into_response())
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
