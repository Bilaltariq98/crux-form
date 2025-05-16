use crate::{
    app::{Event, Model},
    events::form::FieldIdent,
};
use crux_core::{render::render, Command};
use crux_http::{command::Http, HttpError, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AddressSuggestion {
    pub street: String,
    pub city: String,
    pub postcode: String,
    pub country: String,
    pub combined: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AddressSuggestionsResult {
    Success(Vec<AddressSuggestion>),
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AddressEvent {
    FetchSuggestions { query: String },
    SuggestionsReceived(AddressSuggestionsResult),
    SelectSuggestion { suggestion: AddressSuggestion },
    ClearSuggestions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddressHandler {
    suggestions: Vec<AddressSuggestion>,
    api_url: String,
}

impl AddressHandler {
    pub fn new(api_url: String) -> Self {
        Self {
            suggestions: Vec::new(),
            api_url,
        }
    }

    pub fn handle_fetch_suggestions(
        &mut self,
        query: String,
    ) -> Command<crate::app::Effect, crate::app::Event> {
        Http::get(format!("{}?query={}", self.api_url, query))
            .expect_json()
            .build()
            .then_send(
                |result: Result<Response<Vec<AddressSuggestion>>, HttpError>| {
                    Event::SuggestionsReceived(match result {
                        Ok(mut response) => {
                            if let Some(suggestions) = response.take_body() {
                                AddressSuggestionsResult::Success(suggestions)
                            } else {
                                AddressSuggestionsResult::Error
                            }
                        }
                        Err(_) => AddressSuggestionsResult::Error,
                    })
                },
            )
    }

    pub fn handle_suggestions_received(
        &mut self,
        result: AddressSuggestionsResult,
    ) -> Command<crate::app::Effect, crate::app::Event> {
        match result {
            AddressSuggestionsResult::Success(suggestions) => {
                self.suggestions = suggestions;
            }
            AddressSuggestionsResult::Error => {
                self.suggestions.clear();
            }
        }
        render()
    }

    pub fn handle_select_suggestion(
        &mut self,
        suggestion: AddressSuggestion,
    ) -> Command<crate::app::Effect, crate::app::Event> {
        self.suggestions.clear();
        Command::event(Event::UpdateValue {
            ident: FieldIdent::Address,
            value: suggestion.combined.clone(),
        })
        .then(render())
    }

    pub fn handle_clear_suggestions(&mut self) -> Command<crate::app::Effect, crate::app::Event> {
        self.suggestions.clear();
        render()
    }

    pub fn get_suggestions(&self) -> &[AddressSuggestion] {
        &self.suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Effect;

    const API_URL: &str = "http://localhost:8000/api/suggestions";

    #[test]
    fn test_address_handler_fetch_suggestions() {
        let mut handler = AddressHandler::new(API_URL.to_string());
        let mut cmd = handler.handle_fetch_suggestions("test".to_string());
        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Http(_)));
    }

    #[test]
    fn test_address_handler_suggestions_received() {
        let mut handler = AddressHandler::new(API_URL.to_string());
        let suggestions = vec![AddressSuggestion {
            street: "123 Test St".to_string(),
            city: "London".to_string(),
            postcode: "SW1A 1AA".to_string(),
            country: "UK".to_string(),
            combined: "123 Test St, London, SW1A 1AA, UK".to_string(),
        }];
        let mut cmd = handler
            .handle_suggestions_received(AddressSuggestionsResult::Success(suggestions.clone()));
        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Render(_)));
        assert_eq!(handler.get_suggestions(), suggestions);
    }

    #[test]
    fn test_address_handler_clear_suggestions() {
        let mut handler = AddressHandler::new(API_URL.to_string());
        let _ = handler.handle_suggestions_received(AddressSuggestionsResult::Success(vec![
            AddressSuggestion {
                street: "123 Test St".to_string(),
                city: "London".to_string(),
                postcode: "SW1A 1AA".to_string(),
                country: "UK".to_string(),
                combined: "123 Test St, London, SW1A 1AA, UK".to_string(),
            },
        ]));
        let mut cmd = handler.handle_clear_suggestions();
        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Render(_)));
        assert!(handler.get_suggestions().is_empty());
    }
}
