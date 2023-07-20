# nostress
Nostr -> RSS

## Run

`cargo run`

```
nostress --help      (cli-relays)nostress
Usage: nostress [OPTIONS]

Options:
  -b, --bind-addr <BIND_ADDR>               [default: 127.0.0.1:3000]
  -d, --default-relay [<DEFAULT_RELAY>...]
  -h, --help                                Print help
```

`--default-relay` can be specified multiple times. This provides a list of relays to query in case a NIP-05 address doesn't include a relay list.

## Request

`curl localhost:3000/users/_@philipcristiano.com/rss`
