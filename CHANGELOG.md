# Changelog

## [1.1.0] – 2025-06-05
### Changed
- Migrated Vault HTTP client implementation to fully support Hyper v1 ecosystem:
    - Replaced deprecated hyper::Client usage with hyper_util::client::legacy::Client.
    - Used TokioExecutor and TokioTimer for async execution and connection pooling.
    - Updated request construction to use http_body_util::Empty<Bytes> for empty request bodies.

### Added
- HTTPS connector now built with hyper_rustls::HttpsConnectorBuilder using system WebPKI roots.
- Pool timeout and idle timeout settings applied via pool_timer and pool_idle_timeout.
- Inline documentation (///) added to the health_check method, describing Vault status codes and usage examples.
