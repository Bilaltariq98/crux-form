use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressSuggestion {
    pub street: String,
    pub city: String,
    pub postcode: String,
    pub country: String,
    pub combined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AddressSuggestionsResult {
    Success(Vec<AddressSuggestion>),
    Error,
}
