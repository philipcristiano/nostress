use nostr_sdk::prelude::*;
use rss::{Item, ItemBuilder, Guid};

pub fn event_to_item(e: Event) -> Item {
    let c = e.content;
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
    println!("{:?}", &e);
    e.tags.iter().any(|tag|
        match tag.kind() {
            TagKind::P => true,
            TagKind::E => false,
            _ => return false,
        }

    )
}

pub fn filter_out_replies(v: Vec<Event>) -> Vec<Event> {
    return v.into_iter().filter( |e| !event_is_reply(&e)).collect();
}

#[cfg(test)]
mod test_events {

    const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";
    const OTHER_BECH32_SK: &str = "npub1pk0mqgyksddqz2fscyxnxuhfmxkcj36zvq3mcjncyx3lnjals2mqaz3j2a";

    use nostr_sdk::prelude::*;
    use super::filter_out_replies;

    #[test]
    fn include_text_notes_without_tags() {
        let secret_key = SecretKey::from_bech32(BECH32_SK).unwrap();
        let my_keys = Keys::new(secret_key);
        let e_note: Event = EventBuilder::new_text_note("POW text note from nostr-sdk", &[]).to_event(&my_keys).unwrap();

        let notes = vec!(e_note);
        let filtered_notes = filter_out_replies(notes);
        assert_eq!(filtered_notes.len(), 1);
    }

    #[test]
    fn include_text_notes_referencing_another_event_without_pubkeys() {
        let secret_key = SecretKey::from_bech32(BECH32_SK).unwrap();
        let my_keys = Keys::new(secret_key);
        let event_id = EventId::from_hex("b3e392b11f5d4f28321cedd09303a748acfd0487aea5a7450b3481c60b6e4f87").unwrap();
        let tags = [
            Tag::Event(event_id, Some(UncheckedUrl::from("wss://relay.example.com")), None),
        ];
        let e_note: Event = EventBuilder::new_text_note("Text note from nostr-sdk", &tags).to_event(&my_keys).unwrap();

        let notes = vec!(e_note);
        let filtered_notes = filter_out_replies(notes);
        assert_eq!(filtered_notes.len(), 1);
    }

    #[test]
    fn exclude_text_notes_referencing_pubkeys() {
        let secret_key = SecretKey::from_bech32(BECH32_SK).unwrap();
        let my_keys = Keys::new(secret_key);
        let other_pub_key = XOnlyPublicKey::from_bech32(OTHER_BECH32_SK).unwrap();

        let tags = [
            Tag::PubKey(other_pub_key, None),
        ];
        let e_note: Event = EventBuilder::new_text_note("Text note from nostr-sdk", &tags).to_event(&my_keys).unwrap();

        let notes = vec!(e_note);
        let filtered_notes = filter_out_replies(notes);
        assert_eq!(filtered_notes.len(), 0);
    }
}
