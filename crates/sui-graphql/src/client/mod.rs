//! GraphQL client for Sui blockchain.

pub(crate) mod chain;
pub(crate) mod checkpoints;
pub(crate) mod coins;
pub(crate) mod dynamic_fields;
pub(crate) mod execution;
pub(crate) mod objects;
pub(crate) mod transactions;

use reqwest::Url;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wiremock::http::HeaderMap;

use crate::error::Error;
use crate::error::GraphQLError;
use crate::response::Response;

/// GraphQL client for Sui blockchain.
#[derive(Clone, Debug)]
pub struct Client {
    endpoint: Url,
    http: reqwest::Client,
    headers: reqwest::header::HeaderMap,
}

impl Client {
    /// URL for the Sui Foundation provided GraphQL service for mainnet.
    pub const MAINNET: &str = "https://graphql.mainnet.sui.io/graphql";

    /// URL for the Sui Foundation provided GraphQL service for testnet.
    pub const TESTNET: &str = "https://graphql.testnet.sui.io/graphql";

    /// URL for the Sui Foundation provided GraphQL service for devnet.
    pub const DEVNET: &str = "https://graphql.devnet.sui.io/graphql";

    /// Create a new GraphQL client with the given endpoint.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sui_graphql::Client;
    ///
    /// let client = Client::new(Client::MAINNET).unwrap();
    /// ```
    pub fn new(endpoint: &str) -> Result<Self, Error> {
        let endpoint = Url::parse(endpoint)?;
        Ok(Self {
            endpoint,
            http: reqwest::Client::new(),
            headers: reqwest::header::HeaderMap::new(),
        })
    }

    /// Attach a set of headers to be sent with every GraphQL request.
    ///
    /// Replaces any previously-configured headers. Useful for API keys,
    /// authorization tokens, tracing headers, and similar.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sui_graphql::Client;
    ///
    /// let mut headers = reqwest::header::HeaderMap::new();
    /// headers.insert(
    ///     reqwest::header::AUTHORIZATION,
    ///     "Bearer my-token".parse().unwrap(),
    /// );
    ///
    /// let client = Client::new(Client::MAINNET).unwrap().with_headers(headers);
    /// ```
    pub fn with_headers(mut self, headers: reqwest::header::HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Execute a GraphQL query and return the response.
    ///
    /// The response contains both data and any errors (GraphQL supports partial success).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde::Deserialize;
    /// use sui_graphql::Client;
    ///
    /// #[derive(Deserialize)]
    /// struct MyResponse {
    ///     #[serde(rename = "chainIdentifier")]
    ///     chain_identifier: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), sui_graphql::Error> {
    ///     let client = Client::new(Client::MAINNET)?;
    ///     let response = client
    ///         .query::<MyResponse>("query { chainIdentifier }", serde_json::json!({}))
    ///         .await?;
    ///
    ///     // Check for partial errors
    ///     if response.has_errors() {
    ///         for err in response.errors() {
    ///             eprintln!("GraphQL error: {}", err.message());
    ///         }
    ///     }
    ///
    ///     // Access the data
    ///     if let Some(data) = response.data() {
    ///         println!("Chain: {}", data.chain_identifier);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn query<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<Response<T>, Error> {
        #[derive(Serialize)]
        struct GraphQLRequest<'a> {
            query: &'a str,
            variables: serde_json::Value,
        }

        #[derive(Deserialize)]
        struct GraphQLResponse<T> {
            data: Option<T>,
            errors: Option<Vec<GraphQLError>>,
        }

        let request = GraphQLRequest { query, variables };

        let mut req = self.http.post(self.endpoint.clone()).json(&request);
        if !self.headers.is_empty() {
            req = req.headers(self.headers.clone());
        }

        let raw: GraphQLResponse<T> = req.send().await?.json().await?;

        Ok(Response::new(raw.data, raw.errors.unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new() {
        let client = Client::new("https://example.com/graphql").unwrap();
        assert_eq!(client.endpoint.as_str(), "https://example.com/graphql");
    }

    #[test]
    fn test_client_new_invalid_url() {
        let result = Client::new("not a valid url");
        assert!(matches!(result, Err(Error::InvalidUrl(_))));
    }
}
