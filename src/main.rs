use axum::{
    routing::get,
    extract::{Path, State, Query},
    http::{StatusCode, header},
    response::IntoResponse,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use std::ops::Sub;
use rss::{ChannelBuilder, Item};
use clap::Parser;

use nostr_sdk::prelude::*;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "127.0.0.1:3000")]
    bind_addr: String,
    #[arg(short, long, num_args=0.. )]
    default_relay: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
struct NostressConfig {
    default_relays: Vec<String>
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let default_relays = args.default_relay.unwrap_or_default();

    let nostress_config = NostressConfig {
        default_relays
    };
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/users/:user_id/rss", get(user_rss)).with_state(nostress_config);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr: SocketAddr = args.bind_addr.parse().expect("Expected bind addr");
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Debug, Deserialize)]
struct TextNoteFilters {
    include_text_note_replies: Option<bool>,
}

async fn user_rss(State(nc): State<NostressConfig>, Path(user_id): Path<String>, Query(text_note_filters): Query<TextNoteFilters>) -> impl IntoResponse {

    let profile = nostr_sdk::nips::nip05::get_profile(&user_id, None).await.unwrap();

    let profile_key = Keys::from_public_key(profile.public_key);
    let client = Client::new(&profile_key);
    for r in profile.relays {
        client.add_relay(r, None).await.unwrap();
    }
    for default_relay in nc.default_relays {
        client.add_relay(default_relay, None).await.unwrap();
    }

    client.connect().await;

    let now = Timestamp::now();
    let since = now.sub(Duration::from_secs(86400));
    let subscription = Filter::new()
        .author(profile.public_key.to_string())
        .kind(Kind::TextNote)
        .since(since);

    let timeout = Duration::from_secs(10);
    let events = client
        .get_events_of(vec![subscription], Some(timeout))
        .await
        .unwrap();
    client.disconnect().await.unwrap();

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

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/rss+xml")], channel.to_string())

}
