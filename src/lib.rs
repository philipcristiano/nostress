use nostr_sdk::prelude::*;
use rss::{Item, ItemBuilder, Guid};

pub fn event_to_item(e: Event) -> Item {
    let c = e.content;
    let mut guid = Guid::default();
    let event_bech32 = e.id.to_bech32().unwrap();
    let event_link = format!("https://snort.social/e/{event_bech32}");
    guid.set_value(e.id.to_string());

    println!("Author {:?}", e.pubkey);
    println!("Tags {:?}", e.tags);
    ItemBuilder::default()
        .content(c)
        .guid(guid)
        .link(event_link)
        .pub_date(e.created_at.to_human_datetime())
        .build()
}
