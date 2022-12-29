# stac-async

Asynchronous support for [stac-rs](https://github.com/gadomski/stac-rs).

## Usage

This project might release on occasion, but you'll probably just want to use the version directly from Github:

```toml
[dependencies]
stac-async = { git = "https://github.com/gadomski/stac-incubator-rs" }
```

It's really just a thin wrapper around `reqwest::Client`, so there's not much to it:

```rust
let client = stac_async::Client::new();
let item = client.get("http://stac-rs.text/item.json").await.unwrap();
```

There's a top-level function if you only need to use it once, or you're only accessing the local filesystem:

```rust
let item = stac_async::read("data/simple-item.json").await.unwrap();
```
