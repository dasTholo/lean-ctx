//! Local supersession helpers.

use lean_ctx_protocol::{KnowledgeObjectV1, ValidityWindow};

/// Link an existing object to the object that replaces it.
pub fn mark_superseded(object: &mut KnowledgeObjectV1, replacement_id: impl Into<String>) {
    let replacement_id = replacement_id.into();
    let validity = object.validity.get_or_insert_with(|| ValidityWindow {
        valid_from: String::new(),
        valid_until: None,
        superseded_by: None,
    });
    validity.superseded_by = Some(replacement_id);
}

/// Return whether an object has a replacement in its validity metadata.
pub fn is_superseded(object: &KnowledgeObjectV1) -> bool {
    object
        .validity
        .as_ref()
        .is_some_and(|validity| validity.superseded_by.is_some())
}
