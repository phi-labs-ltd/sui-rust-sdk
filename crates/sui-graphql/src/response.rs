//! Response type for GraphQL queries.

use reqwest::StatusCode;

use crate::error::GraphQLError;

/// A GraphQL response containing data and/or errors.
///
/// GraphQL responses can have three states:
/// - Success: `data` is present, `errors` is empty
/// - Partial success: `data` is present AND `errors` is non-empty
/// - Failure: `data` is None, `errors` is non-empty
///
/// None of the three implies a particular HTTP status. A server may report query-level errors
/// under a non-success status while still returning a usable response, so [`Response::status`]
/// stays available alongside the data.
#[derive(Debug)]
pub struct Response<T> {
    status: StatusCode,
    data: Option<T>,
    errors: Vec<GraphQLError>,
}

impl<T> Response<T> {
    /// Create a new response with data and errors.
    pub(crate) fn new(status: StatusCode, data: Option<T>, errors: Vec<GraphQLError>) -> Self {
        Self {
            status,
            data,
            errors,
        }
    }

    /// HTTP status the response arrived under.
    ///
    /// Usually a success status. A GraphQL server that reports query-level errors with a
    /// non-success status produces a response that is still worth reading, and this is the only
    /// place that status survives.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The deserialized data from the response, if present.
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    /// Consumes the response and returns the data, if present.
    pub fn into_data(self) -> Option<T> {
        self.data
    }

    /// Returns true if the response has any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns all errors from the response.
    pub fn errors(&self) -> &[GraphQLError] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_error() -> GraphQLError {
        serde_json::from_value(serde_json::json!({
            "message": "test error",
            "path": ["field"]
        }))
        .unwrap()
    }

    #[test]
    fn test_response_no_errors() {
        let response: Response<String> =
            Response::new(StatusCode::OK, Some("data".to_string()), vec![]);
        assert!(!response.has_errors());
        assert!(response.errors().is_empty());
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_response_with_errors() {
        let response: Response<String> =
            Response::new(StatusCode::OK, Some("data".to_string()), vec![make_error()]);
        assert!(response.has_errors());
        assert_eq!(response.errors().len(), 1);
    }
}
