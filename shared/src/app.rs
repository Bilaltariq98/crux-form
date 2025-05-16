use crux_core::{macros::effect, render::RenderOperation, Command};
use crux_http::protocol::HttpRequest;
use serde::{Deserialize, Serialize};

use crate::events::address::{AddressHandler, AddressSuggestion};
use crate::events::form::{FormHandler, FormViewModel};

const ADDRESS_API_URL: &str = "http://localhost:8000/api/suggestions";

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    form_handler: FormHandler,
    address_handler: AddressHandler,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            form_handler: FormHandler::new(),
            address_handler: AddressHandler::new(ADDRESS_API_URL.to_string()),
        }
    }
}

// Main ViewModel
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewModel {
    pub form: FormViewModel,
    pub address_suggestions: Vec<AddressSuggestion>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    // Form events
    UpdateValue {
        ident: crate::events::form::FieldIdent,
        value: String,
    },
    TouchField {
        ident: crate::events::form::FieldIdent,
    },
    SetFieldEditing {
        ident: crate::events::form::FieldIdent,
        editing: bool,
    },
    Submit,
    Edit,
    ResetForm,

    // Address events
    FetchSuggestions {
        query: String,
    },
    SuggestionsReceived(crate::events::address::AddressSuggestionsResult),
    SelectSuggestion {
        suggestion: AddressSuggestion,
    },
    ClearSuggestions,
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
}

#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Model = Model;
    type Event = Event;
    type ViewModel = ViewModel;
    type Capabilities = ();
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model, _caps: &()) -> Command<Effect, Event> {
        match event {
            // Form events
            Event::UpdateValue { ident, value } => {
                model.form_handler.handle_update_value(ident, value)
            }
            Event::TouchField { ident } => model.form_handler.handle_touch_field(ident),
            Event::SetFieldEditing { ident, editing } => {
                model.form_handler.handle_set_field_editing(ident, editing)
            }
            Event::Submit => model.form_handler.handle_submit(),
            Event::Edit => model.form_handler.handle_edit(),
            Event::ResetForm => model.form_handler.handle_reset(),

            // Address events
            Event::FetchSuggestions { query } => {
                model.address_handler.handle_fetch_suggestions(query)
            }
            Event::SuggestionsReceived(result) => {
                model.address_handler.handle_suggestions_received(result)
            }
            Event::SelectSuggestion { suggestion } => {
                model.address_handler.handle_select_suggestion(suggestion)
            }
            Event::ClearSuggestions => model.address_handler.handle_clear_suggestions(),
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            form: model.form_handler.view(),
            address_suggestions: model.address_handler.get_suggestions().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::form::FieldIdent;
    use crux_core::App as _;

    #[test]
    fn test_update_value() {
        let app = App::default();
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "TestUser".to_string(),
            },
            &mut model,
            &(),
        );

        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Render(_)));
        assert_eq!(
            model.form_handler.get_form().username.value,
            "TestUser".into()
        );
    }

    #[test]
    fn test_fetch_suggestions() {
        let app = App::default();
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::FetchSuggestions {
                query: "test".to_string(),
            },
            &mut model,
            &(),
        );

        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Http(_)));
    }

    #[test]
    fn test_view_model() {
        let app = App::default();
        let model = Model::default();
        let view = app.view(&model);

        assert_eq!(view.form.username.value, "");
        assert!(!view.form.submitted);
        assert!(view.form.is_editing_form);
        assert!(view.address_suggestions.is_empty());
    }
}
