use crate::limit::RateLimit;
use crate::query::ResourceComponent;
use crate::response::{Failures, MessageType};
use std::result;
use thiserror::Error as ThisError;

/// A type alias for handling errors throughout crossref.
pub type Result<T> = result::Result<T, Error>;

/// All different error types this crate uses.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// if an invalid type was requested
    #[error("invalid type name: {name}")]
    InvalidTypeName {
        /// the type name that was invalid
        name: String,
    },

    /// if there is a mismatch between the expected return type of the crossref api and this rust client
    #[error("expected response item of type {expected} but got {got}")]
    UnexpectedItem {
        /// the message type this client expected
        expected: MessageType,
        /// the message type crossref actually returned
        got: MessageType,
    },

    /// a filter value crossref has no way to read back
    ///
    /// Crossref splits the `filter` value on `,` after percent-decoding it, so
    /// a value carrying one arrives as two filters -- `container-title:A, B`
    /// becomes `container-title:A` plus a filter called ` B`. There is no form
    /// that survives the split, so the request is refused rather than sent to
    /// be rejected with a `400`.
    #[error(
        "`{value}` cannot be sent as a `{filter}` filter: crossref reads the `,` in it as the start of another filter"
    )]
    UnsendableFilterValue {
        /// the filter whose value cannot be sent, e.g. `container-title`
        filter: String,
        /// the value that carries the `,`
        value: String,
    },

    /// a config error
    #[error("{msg}")]
    Config {
        /// the notification
        msg: String,
    },

    /// an error that occurred while operating with [reqwest]
    #[error(transparent)]
    ReqWest {
        /// the underlying transport error
        #[from]
        reqwest: reqwest::Error,
    },

    /// crossref refused the request and said why
    ///
    /// Answered with a `400` and a `validation-failure` body, which is what an
    /// unknown filter, sort field or field query produces.
    #[error("crossref rejected the request: {failures}")]
    ValidationFailure {
        /// what crossref objected to
        failures: Failures,
    },

    /// crossref kept answering `429` until the retry budget ran out
    ///
    /// The client paces itself against the limit crossref reports, so this
    /// normally means requests are also being made outside it -- from another
    /// process, or another [`Crossref`](crate::Crossref) built separately
    /// rather than cloned.
    #[error("crossref rate limited the request; gave up after {attempts} attempts")]
    RateLimited {
        /// how many times the request was sent
        attempts: u32,
        /// the budget crossref last reported
        limit: RateLimit,
    },

    /// When crossref could not find anything
    #[error("Nothing was found for resource `{resource}`")]
    ResourceNotFound {
        /// the resource that could not be resolved
        resource: Box<ResourceComponent>,
    },

    /// if a error in serde occurred
    #[error("invalid serde: {error}")]
    Serde {
        /// the underlying (de)serialization error
        #[from]
        error: serde_json::Error,
    },
}
