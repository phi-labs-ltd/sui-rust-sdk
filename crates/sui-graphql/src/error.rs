//! Error types for the GraphQL client.

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Error type for GraphQL client operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP or network error.
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    /// Invalid URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// The endpoint answered with a non-success HTTP status and a body that is not a GraphQL
    /// response. Proxies, load balancers and API gateways in front of an endpoint report their
    /// own failures this way, in HTML, plain text or an unrelated JSON shape.
    #[error("HTTP status {status}: {body}")]
    HttpStatus {
        /// HTTP status of the response.
        status: reqwest::StatusCode,
        /// The response body verbatim, so a caller can parse whatever the intermediary reported.
        /// Not truncated: the body is fully buffered before this error is built either way, and
        /// clipping it would only cost information — a JSON error body would stop being parsable.
        /// Lossily decoded, since an error page need not be valid UTF-8.
        body: String,
    },

    /// The server rejected the request and returned no data.
    ///
    /// GraphQL reports this as a non-empty `errors` list with `data` absent or null, and it is
    /// independent of the HTTP status: a server may answer with `200` and
    /// `application/json`, or with `400` and `application/graphql-response+json`. Both mean the
    /// query never ran, so the distinction is not the caller's to make.
    ///
    /// Partial success — some `data` alongside `errors` — is not this error; it arrives as
    /// [`Response`](crate::Response) so the caller can decide what the partial data is worth.
    #[error(
        "GraphQL request failed under HTTP {status}: {}",
        join_messages(errors)
    )]
    GraphQL {
        /// HTTP status the errors arrived under.
        status: reqwest::StatusCode,
        /// The errors reported by the server, never empty.
        errors: Vec<GraphQLError>,
    },

    /// Failed to serialize data (e.g., BCS encoding).
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Failed to deserialize or decode response data.
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Missing expected data in response.
    #[error("Missing expected data: {0}")]
    MissingData(&'static str),
}

/// Renders every error message into one line, for [`Error::GraphQL`]'s `Display`.
fn join_messages(errors: &[GraphQLError]) -> String {
    errors
        .iter()
        .map(GraphQLError::message)
        .collect::<Vec<_>>()
        .join("; ")
}

impl From<base64ct::Error> for Error {
    fn from(err: base64ct::Error) -> Self {
        Self::Deserialization(format!("base64 decode: {err}"))
    }
}

impl From<bcs::Error> for Error {
    fn from(err: bcs::Error) -> Self {
        Self::Deserialization(format!("bcs decode: {err}"))
    }
}

impl From<sui_sdk_types::TypeParseError> for Error {
    fn from(err: sui_sdk_types::TypeParseError) -> Self {
        Self::Deserialization(format!("type parse: {err}"))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::Deserialization(format!("integer parse: {err}"))
    }
}

impl From<sui_sdk_types::DigestParseError> for Error {
    fn from(err: sui_sdk_types::DigestParseError) -> Self {
        Self::Deserialization(format!("digest parse: {err}"))
    }
}

// =============================================================================
// GraphQL Response Types (per GraphQL spec)
// https://spec.graphql.org/October2021/#sec-Errors
// =============================================================================

/// A single GraphQL error.
#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    message: String,
    locations: Option<Vec<Location>>,
    path: Option<Vec<PathFragment>>,
    extensions: Option<HashMap<String, serde_json::Value>>,
}

impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl GraphQLError {
    /// A description of the error.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Locations in the query where the error occurred.
    pub fn locations(&self) -> Option<&[Location]> {
        self.locations.as_deref()
    }

    /// Path to the field that caused the error (e.g., `["user", "name"]`).
    pub fn path(&self) -> Option<&[PathFragment]> {
        self.path.as_deref()
    }

    /// Additional error metadata (e.g., error code).
    pub fn extensions(&self) -> Option<&HashMap<String, serde_json::Value>> {
        self.extensions.as_ref()
    }

    /// Attempt to extract an error code from the extensions, if present.
    pub fn code(&self) -> Option<&str> {
        self.extensions.as_ref()?.get("code")?.as_str()
    }
}

/// A segment in an error path - either a field name or array index.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PathFragment {
    /// Field name in the response.
    Key(String),
    /// Array index in the response.
    Index(i32),
}

/// Location in the GraphQL query where an error occurred.
#[derive(Debug, Deserialize)]
pub struct Location {
    /// Line number (1-indexed).
    pub line: i32,
    /// Column number (1-indexed).
    pub column: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        // Use a relative URL (no scheme) to trigger a builder error
        let err = Error::Request(
            reqwest::Client::new()
                .get("not a valid url")
                .build()
                .unwrap_err(),
        );
        assert!(err.to_string().contains("Request error"));
    }

    #[test]
    fn graphql_error_display_carries_status_and_every_message() {
        let errors = ["first thing broke", "second thing broke"]
            .into_iter()
            .map(|message| serde_json::from_value(serde_json::json!({ "message": message })))
            .collect::<Result<Vec<GraphQLError>, _>>()
            .unwrap();

        let err = Error::GraphQL {
            status: reqwest::StatusCode::BAD_REQUEST,
            errors,
        };

        let rendered = err.to_string();
        assert!(rendered.contains("400"), "{rendered}");
        assert!(rendered.contains("first thing broke"), "{rendered}");
        assert!(rendered.contains("second thing broke"), "{rendered}");
    }

    #[test]
    fn test_graphql_error_display() {
        let err: GraphQLError =
            serde_json::from_value(serde_json::json!({"message": "Field not found"})).unwrap();
        assert_eq!(err.to_string(), "Field not found");
    }

    #[test]
    fn test_graphql_error_code() {
        let err: GraphQLError = serde_json::from_value(serde_json::json!({
            "message": "Unknown field \"foo\" on type \"Query\".",
            "extensions": {"code": "GRAPHQL_VALIDATION_FAILED"}
        }))
        .unwrap();
        assert_eq!(err.code(), Some("GRAPHQL_VALIDATION_FAILED"));
    }

    #[test]
    fn test_graphql_error_no_code() {
        let err: GraphQLError =
            serde_json::from_value(serde_json::json!({"message": "Error"})).unwrap();
        assert_eq!(err.code(), None);
    }

    #[test]
    fn test_path_fragment_deserialization() {
        let json = r#"["object", 0, "field"]"#;
        let path: Vec<PathFragment> = serde_json::from_str(json).unwrap();

        assert!(matches!(&path[0], PathFragment::Key(k) if k == "object"));
        assert!(matches!(&path[1], PathFragment::Index(0)));
        assert!(matches!(&path[2], PathFragment::Key(k) if k == "field"));
    }
}
