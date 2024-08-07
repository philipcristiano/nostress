use nostr_sdk::prelude::*;
use rss::{Guid, Item, ItemBuilder};

pub fn event_to_item(e: Event) -> Item {
    let c = linkify_content(e.content.clone());
    let mut guid = Guid::default();
    let event_bech32 = e.id.to_bech32().unwrap();
    let event_link = format!("https://snort.social/e/{event_bech32}");
    guid.set_value(e.id.to_string());

    ItemBuilder::default()
        .content(c)
        .guid(guid)
        .link(event_link)
        .pub_date(e.created_at.to_human_datetime())
        .build()
}

fn event_is_reply(e: &Event) -> bool {
    e.tags.iter().any(|tag| tag.is_reply())
}

pub fn linkify_content(content: String) -> String {
    let finder = linkify::LinkFinder::new();
    let links: Vec<_> = finder.links(&content).collect();
    let mut new_content = content.clone();

    for l in links {
        let link_str = l.as_str();
        let anchor = format!("<a href=\"{link_str}\">{link_str}</a>");
        new_content = new_content.replace(link_str, &anchor);
    }
    new_content
}

pub fn filter_out_replies(v: Vec<Event>) -> Vec<Event> {
    return v.into_iter().filter(|e| !event_is_reply(&e)).collect();
}

#[cfg(test)]
mod test_events {

    const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";
    const OTHER_BECH32_SK: &str = "npub1pk0mqgyksddqz2fscyxnxuhfmxkcj36zvq3mcjncyx3lnjals2mqaz3j2a";

    use super::filter_out_replies;
    use nostr_sdk::prelude::*;

    #[test]
    fn include_text_notes_without_tags() {
        let secret_key = SecretKey::from_bech32(BECH32_SK).unwrap();
        let my_keys = Keys::new(secret_key);
        let e_note: Event = EventBuilder::text_note("POW text note from nostr-sdk", [])
            .to_event(&my_keys)
            .unwrap();

        let notes = vec![e_note];
        let filtered_notes = filter_out_replies(notes);
        assert_eq!(filtered_notes.len(), 1);
    }

    #[test]
    fn include_text_notes_referencing_another_event_without_pubkeys() {
        let secret_key = SecretKey::from_bech32(BECH32_SK).unwrap();
        let my_keys = Keys::new(secret_key);
        let event_id =
            EventId::from_hex("b3e392b11f5d4f28321cedd09303a748acfd0487aea5a7450b3481c60b6e4f87")
                .unwrap();
        let t = Tag::event(event_id);
        let e_note: Event = EventBuilder::text_note("Text note from nostr-sdk", [t])
            .to_event(&my_keys)
            .unwrap();

        let notes = vec![e_note];
        let filtered_notes = filter_out_replies(notes);
        assert_eq!(filtered_notes.len(), 1);
    }

    #[test]
    fn exclude_replies() {
        let secret_key = SecretKey::from_bech32(BECH32_SK).unwrap();
        let my_keys = Keys::new(secret_key);
        let other_pub_key = PublicKey::from_bech32(OTHER_BECH32_SK).unwrap();

        let t = Tag::public_key(other_pub_key);

        let original_note: Event = EventBuilder::text_note("Text note from nostr-sdk", [t])
            .to_event(&my_keys)
            .unwrap();
        let e_note: Event =
            EventBuilder::text_note_reply("Text note from nostr-sdk", &original_note, None, None)
                .to_event(&my_keys)
                .unwrap();

        let notes = vec![e_note];
        let filtered_notes = filter_out_replies(notes);
        assert_eq!(filtered_notes.len(), 0);
    }
}
