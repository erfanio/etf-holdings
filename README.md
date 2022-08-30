# ETF Holdings Backend

- `lib` provides the brains of digging up the holding of ETFs. The main API is `AvailableETFs`.
- `web-server` presents the raw details information in a more usable way (incl. fetching price data for ETFs and holdings to draw a price chart).

## Docs
Cargo's built-in docs work great for exploring this package

```
$ cargo doc --document-private-items --open
```

## Running

```
$ cargo run
```
