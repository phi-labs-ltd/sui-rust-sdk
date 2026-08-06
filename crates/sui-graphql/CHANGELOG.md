# [0.3.2] - 2026-08-06

## Added
- `Error::HttpStatus`, carrying the HTTP status and the response body verbatim
- re-exports `reqwest::StatusCode`, named by the above, so callers do not need a
  direct reqwest dependency

## Changed
- `Client::query` now checks the HTTP status before decoding the body, and
  returns `Error::HttpStatus` for a non-success status. Previously the body was
  decoded as GraphQL data regardless: an intermediary's HTML or empty error page
  became a `reqwest` decode error with the status discarded, and an unrelated
  JSON error body became `Ok` with no data, indistinguishable from a query that
  legitimately returned nothing

# [0.3.1] - 2026-07-16

## Added
- [#244] custom HTTP headers on `Client` via `with_headers` and
  `extend_headers`, with `bearer_auth` and `basic_auth` helpers; re-exports
  `reqwest::header` so callers do not need a direct reqwest dependency

## Changed
- [#273] every request now sends a `client-sdk-type: rust` header for server
  metrics, overriding any caller-supplied value for that header

[#244]: https://github.com/MystenLabs/sui-rust-sdk/pull/244
[#273]: https://github.com/MystenLabs/sui-rust-sdk/pull/273

# [0.3.0] - 2026-03-23

Initial published release.

## Added
- [#191] basic GraphQL client with partial error handling
- [#194] `QueryResponse` derive macro integration for field extraction
- [#195] array extraction support
- [#196] compile-time schema validation
- [#197] field suggestions for validation errors
- [#198] pagination support and object query methods
- [#199] coin balance query methods
- [#200] chain info methods with alias support
- [#205] transaction fetching API
- [#206] checkpoint fetching API
- [#207] dynamic fields API
- [#208] transaction execution API
- [#217] enum support in `#[derive(Response)]` for GraphQL union types
- [#222] backward pagination and additional object query methods
- [#224] crate-level documentation and README

[#191]: https://github.com/MystenLabs/sui-rust-sdk/pull/191
[#194]: https://github.com/MystenLabs/sui-rust-sdk/pull/194
[#195]: https://github.com/MystenLabs/sui-rust-sdk/pull/195
[#196]: https://github.com/MystenLabs/sui-rust-sdk/pull/196
[#197]: https://github.com/MystenLabs/sui-rust-sdk/pull/197
[#198]: https://github.com/MystenLabs/sui-rust-sdk/pull/198
[#199]: https://github.com/MystenLabs/sui-rust-sdk/pull/199
[#200]: https://github.com/MystenLabs/sui-rust-sdk/pull/200
[#205]: https://github.com/MystenLabs/sui-rust-sdk/pull/205
[#206]: https://github.com/MystenLabs/sui-rust-sdk/pull/206
[#207]: https://github.com/MystenLabs/sui-rust-sdk/pull/207
[#208]: https://github.com/MystenLabs/sui-rust-sdk/pull/208
[#217]: https://github.com/MystenLabs/sui-rust-sdk/pull/217
[#222]: https://github.com/MystenLabs/sui-rust-sdk/pull/222
[#224]: https://github.com/MystenLabs/sui-rust-sdk/pull/224

[0.3.1]: https://github.com/mystenlabs/sui-rust-sdk/releases/tag/sui-graphql-0.3.1
[0.3.0]: https://github.com/mystenlabs/sui-rust-sdk/releases/tag/sui-graphql-0.3.0
