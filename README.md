# clickhouse-rs

An efficient, asynchronous connection pool for ClickHouse, built upon the [official ClickHouse Rust client](https://github.com/ClickHouse/clickhouse-rs).

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![License][license-badge]][license-url]

[crates-badge]: https://img.shields.io/crates/v/clickhouse-pool.svg
[crates-url]: https://crates.io/crates/clickhouse-pool
[docs-badge]: https://docs.rs/clickhouse-pool/badge.svg
[docs-url]: https://docs.rs/clickhouse-pool
[license-badge]: https://img.shields.io/github/license/acsandmann/clickhouse-pool
[license-url]: https://github.com/acsandmann/clickhouse-pool/blob/main/LICENSE

## Usage

To use the crate, add this to your `Cargo.toml`:
```toml
[dependencies]
clickhouse = "0.12.2"
clickhouse-pool = "0.0.1"
```