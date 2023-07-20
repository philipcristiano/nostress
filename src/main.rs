use axum::{
    routing::get,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    Router,
};
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
}

#[derive(Clone, Debug)]
struct NostressConfig {
    default_relay: String
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let nostress_config = NostressConfig {
        default_relay: "wss://relay.damus.io".to_string()
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

async fn user_rss(State(nc): State<NostressConfig>, Path(user_id): Path<String>) -> impl IntoResponse {

    let profile = nostr_sdk::nips::nip05::get_profile(&user_id, None).await.unwrap();

    let profile_key = Keys::from_public_key(profile.public_key);
    println!("profile: {:?}", profile);
    let client = Client::new(&profile_key);
    for r in profile.relays {
        println!("relay: {r}");
        client.add_relay(r, None).await.unwrap();
    }
    client.add_relay(nc.default_relay, None).await.unwrap();

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

    for e in events {
        items.push(nostress::event_to_item(e));
    }

    channel.set_items(items);

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/rss+xml")], channel.to_string())

}
