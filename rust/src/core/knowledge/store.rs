//! Storage contract for portable local knowledge objects.

use super::query::KnowledgeQuery;
use lean_ctx_protocol::KnowledgeObjectV1;
use std::error::Error;
use std::fmt;

/// Errors returned by a Knowledge Hub store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeStoreError {
    /// The content-addressed object has no usable identifier.
    EmptyObjectId,
    /// A supersession requested an object that is not present.
    NotFound(String),
    /// The object failed protocol validation.
    InvalidObject(String),
}

impl fmt::Display for KnowledgeStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObjectId => formatter.write_str("knowledge object id must not be empty"),
            Self::NotFound(object_id) => {
                write!(formatter, "knowledge object not found: {object_id}")
            }
            Self::InvalidObject(message) => {
                write!(formatter, "invalid knowledge object: {message}")
            }
        }
    }
}

impl Error for KnowledgeStoreError {}

/// Minimal CRUD and supersession surface shared by local and enterprise stores.
pub trait KnowledgeStore {
    /// Fetch a content-addressed object by its stable object id.
    fn get(&self, object_id: &str) -> Option<KnowledgeObjectV1>;

    /// Insert or replace an object.
    fn put(&mut self, object: KnowledgeObjectV1) -> Result<(), KnowledgeStoreError>;

    /// Return objects matching every supplied filter.
    fn query(&self, query: &KnowledgeQuery) -> Vec<KnowledgeObjectV1>;

    /// Store a replacement and link the old object to it through
    /// `ValidityWindow::superseded_by`.
    fn supersede(
        &mut self,
        old_object_id: &str,
        replacement: KnowledgeObjectV1,
    ) -> Result<(), KnowledgeStoreError>;

    /// Delete an object and report whether it existed.
    fn delete(&mut self, object_id: &str) -> bool;
}

/// Return the stable identifier used by the local store.
pub fn object_id(object: &KnowledgeObjectV1) -> &str {
    object.object_id()
}

/// Validate the protocol-level invariants required before storing an object.
pub fn validate_object(object: &KnowledgeObjectV1) -> Result<(), KnowledgeStoreError> {
    if object_id(object).trim().is_empty() {
        return Err(KnowledgeStoreError::EmptyObjectId);
    }
    object
        .validate()
        .map_err(|error| KnowledgeStoreError::InvalidObject(error.to_string()))
}
