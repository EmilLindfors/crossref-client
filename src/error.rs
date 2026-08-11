use crate::query::ResourceComponent;
use crate::response::MessageType;
use std::result;
use thiserror::Error as ThisError;

/// A type alias for handling errors throughout crossref.
pub type Result<T> = result::Result<T, Error>;

/// All different error types this crate uses.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// if a message type was invalid
    #[error("invalid message type: {name}")]
    InvalidMessageType {
        /// the message type that was invalid
        name: String,
    },

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

    /// a config error
    #[error("{msg}")]
    Config {
        /// the notification
        msg: String,
    },

    /// a field the client requires was absent from the response
    #[error("{msg}")]
    MissingField {
        /// the notification
        msg: String,
    },

    /// a field of the response could not be interpreted
    #[error("{msg}")]
    InvalidField {
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

    /// When no message was found but expected
    #[error("No message found but expected message of type `{expected}`")]
    MissingMessage {
        /// the message type this client expected
        expected: MessageType,
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

    /// a DOI did not have the shape crossref expects
    #[error("{error}")]
    DoiValidationError {
        /// the notification
        error: String,
    },

    /// the client was misused
    #[error("{error}")]
    ClientError {
        /// the notification
        error: String,
    },

    /// a result control could not be parsed
    #[error("{error}")]
    InvalidResultControl {
        /// the notification
        error: String,
    },
}
