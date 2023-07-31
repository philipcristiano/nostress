# nostress
Nostr -> RSS

## Run

`cargo run`

```
nostress --help
Usage: nostress [OPTIONS]

Options:
  -b, --bind-addr <BIND_ADDR>               [default: 127.0.0.1:3000]
  -d, --default-relay [<DEFAULT_RELAY>...]
  -h, --help                                Print help
```

`--default-relay` can be specified multiple times. This provides a list of relays to query in case a NIP-05 address doesn't include a relay list.

## Request

`curl localhost:3000/users/_@philipcristiano.com/rss`

### Options


`?include_text_note_replies` (default: false) - Replies are not included by default, set `?include_text_note_replies=true` to include them in the RSS feed
