use crux_core::{
    macros::effect,
    render::{render, RenderOperation},
    Command,
};
use crux_http::{command::Http, protocol::HttpRequest, HttpError, Response};
use serde::{Deserialize, Serialize};

use crate::field::Field;
use crate::form::Form;
use crate::address::{AddressSuggestion, AddressSuggestionsResult};

const ADDRESS_API_URL: &str = "http://localhost:8000/api/suggestions";

// Helper to identify which field is being updated or interacted with
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum FieldIdent {
    Username,
    Email,
    Age,
    Address,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub form: Form,
    pub address_suggestions: Vec<AddressSuggestion>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            form: Form::default(), // Initialize Form using its Default impl
            address_suggestions: Vec::new(),
        }
    }
}

// ViewModel for individual fields
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldViewModel {
    pub value: String,
    pub initial_value: String,
    pub touched: bool,
    pub dirty: bool,
    pub error: Option<String>,
    pub valid: bool,
    pub editing: bool,
}

// Main ViewModel
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewModel {
    pub username: FieldViewModel,
    pub email: FieldViewModel,
    pub age: FieldViewModel,
    pub address: FieldViewModel,
    pub address_suggestions: Vec<AddressSuggestion>,
    pub submitted: bool,
    pub is_editing_form: bool, // This maps to form.is_editing
    pub status_message: String,
    pub can_submit: bool, // This maps to form.can_submit()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    UpdateValue { ident: FieldIdent, value: String },
    TouchField { ident: FieldIdent },
    SetFieldEditing { ident: FieldIdent, editing: bool },
    Submit,
    Edit,
    ResetForm,
    FetchAddressSuggestions { query: String },
    AddressSuggestionsReceived(AddressSuggestionsResult),
    SelectAddressSuggestion { suggestion: AddressSuggestion },
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
    type Capabilities = (); // Assuming no external capabilities for now
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model, _caps: &()) -> Command<Effect, Event> {
        match event {
            Event::UpdateValue { ident, value } => {
                if !model.form.is_editing {
                    return Command::done();
                }

                match ident {
                    FieldIdent::Username => model.form.username.update_value(value),
                    FieldIdent::Email => model.form.email.update_value(value),
                    FieldIdent::Age => {
                        model.form.age.update_value(value.parse::<u32>().ok());
                    }
                    FieldIdent::Address => {
                        model.form.address.update_value(value.clone());
                        if model.form.is_editing {
                            return Http::get(format!("{}?query={}", ADDRESS_API_URL, value))
                                .expect_json()
                                .build()
                                .then_send(|result: Result<Response<Vec<AddressSuggestion>>, HttpError>| {
                                    Event::AddressSuggestionsReceived(match result {
                                        Ok(mut response) => {
                                            if let Some(suggestions) = response.take_body() {
                                                AddressSuggestionsResult::Success(suggestions)
                                            } else {
                                                AddressSuggestionsResult::Error
                                            }
                                        }
                                        Err(_) => AddressSuggestionsResult::Error,
                                    })
                                });
                        }
                    },
                }
                render()
            }
            Event::TouchField { ident } => {
                if !model.form.is_editing {
                    return Command::done();
                }
                match ident {
                    FieldIdent::Username => model.form.username.touch(),
                    FieldIdent::Email => model.form.email.touch(),
                    FieldIdent::Age => model.form.age.touch(),
                    FieldIdent::Address => {
                        model.form.address.touch();
                        model.address_suggestions.clear();
                    },
                }
                render()
            }
            Event::SetFieldEditing { ident, editing } => {
                if !model.form.is_editing && editing {
                    return Command::done();
                }
                match ident {
                    FieldIdent::Username => model.form.username.set_editing(editing),
                    FieldIdent::Email => model.form.email.set_editing(editing),
                    FieldIdent::Age => model.form.age.set_editing(editing),
                    FieldIdent::Address => {
                        model.form.address.set_editing(editing);
                        if !editing {
                            model.address_suggestions.clear();
                        }
                    },
                }
                render()
            }
            Event::FetchAddressSuggestions { query } => {
                if model.form.is_editing {
                    Http::get(format!("{}?query={}", ADDRESS_API_URL, query))
                        .expect_json()
                        .build()
                        .then_send(|result: Result<Response<Vec<AddressSuggestion>>, HttpError>| {
                            Event::AddressSuggestionsReceived(match result {
                                Ok(mut response) => {
                                    if let Some(suggestions) = response.take_body() {
                                        AddressSuggestionsResult::Success(suggestions)
                                    } else {
                                        AddressSuggestionsResult::Error
                                    }
                                }
                                Err(_) => AddressSuggestionsResult::Error,
                            })
                        })
                } else {
                    Command::done()
                }
            }
            Event::AddressSuggestionsReceived(result) => {
                match result {
                    AddressSuggestionsResult::Success(suggestions) => {
                        // Filter out suggestions that exactly match the current address
                        model.address_suggestions = suggestions
                            .into_iter()
                            .filter(|s| s.combined != model.form.address.value)
                            .collect();
                    }
                    AddressSuggestionsResult::Error => {
                        model.address_suggestions.clear();
                    }
                }
                render()
            }
            Event::Submit => {
                model.form.touch_all(); // Mark all fields as touched
                model.form.validate_all(); // Validate all fields

                if model.form.is_valid() {
                    model.form.submitted = true;
                    model.form.set_editing(false); // Form is no longer editable
                    model.address_suggestions.clear(); // Clear suggestions on submit
                } else {
                    model.form.submitted = false; // Ensure submitted is false if validation fails
                                                  // is_editing remains true to allow corrections
                }
                render()
            }
            Event::Edit => {
                // Allow editing the form again
                model.form.submitted = false;
                model.form.set_editing(true);
                model.address_suggestions.clear(); // Clear suggestions when starting to edit

                #[cfg(target_arch = "wasm32")]
                {
                    let log_message = format!("{:?}", model.form);
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&log_message));
                }
                render()
            }
            Event::ResetForm => {
                model.form.reset(); // Resets form to default, sets is_editing to true
                model.address_suggestions.clear(); // Clear suggestions on reset
                render()
            }
            Event::SelectAddressSuggestion { suggestion } => {
                if !model.form.is_editing {
                    return Command::done();
                }
                
                // Update the address value
                model.form.address.update_value(suggestion.combined.clone());
                // Mark the field as touched since it's a valid selection
                model.form.address.touch();
                // Clear suggestions immediately
                model.address_suggestions.clear();
                
                render()
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let map_field_to_view_model = |field: &Field<String>| FieldViewModel {
            value: field.value.clone(),
            initial_value: field.initial_value.clone(),
            touched: field.touched,
            dirty: field.dirty,
            error: field.error.clone(),
            valid: field.valid,
            editing: field.editing,
        };

        let map_option_u32_field_to_view_model = |field: &Field<Option<u32>>| FieldViewModel {
            value: field.value.map_or_else(String::new, |v| v.to_string()),
            initial_value: field
                .initial_value
                .map_or_else(String::new, |v| v.to_string()),
            touched: field.touched,
            dirty: field.dirty,
            error: field.error.clone(),
            valid: field.valid,
            editing: field.editing,
        };

        let username_vm = map_field_to_view_model(&model.form.username);
        let email_vm = map_field_to_view_model(&model.form.email);
        let age_vm = map_option_u32_field_to_view_model(&model.form.age);
        let address_vm = map_field_to_view_model(&model.form.address);

        let status_message = if model.form.submitted {
            "Form Submitted Successfully!".to_string()
        } else if !model.form.is_editing {
            "Form data (View only)".to_string() // After successful submission
        } else if model.form.username.dirty
            || model.form.email.dirty
            || model.form.age.dirty
            || model.form.address.dirty
        {
            // Any field is dirty, form has unsaved changes. This check should ideally be a method on Form.
            // e.g. model.form.is_dirty()
            "Form has unsaved changes".to_string()
        } else {
            // Not submitted, is editing, no fields are dirty
            // Check for validation errors on initial load or after reset
            if !model.form.is_valid() {
                "Please correct the errors.".to_string()
            } else {
                "Please fill out the form.".to_string()
            }
        };

        Self::ViewModel {
            username: username_vm,
            email: email_vm,
            age: age_vm,
            address: address_vm,
            address_suggestions: model.address_suggestions.clone(),
            submitted: model.form.submitted,
            is_editing_form: model.form.is_editing,
            can_submit: model.form.can_submit(),
            status_message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, FieldIdent, App, Model, AddressSuggestion, Effect, ADDRESS_API_URL, AddressSuggestionsResult};
    use crux_core::{App as _};
    use crux_http::{
        protocol::{HttpRequest, HttpResult, HttpResponse},
        HttpError,
    };

    #[test]
    fn initial_state() {
        let app = App::default();
        let model = Model::default(); // Model::default() now initializes form with its defaults
        let view = app.view(&model);

        assert_eq!(view.username.value, "");
        assert!(!view.username.valid);
        assert_eq!(
            view.username.error.as_deref(),
            Some("Username cannot be empty")
        );

        assert_eq!(view.email.value, "");
        assert!(!view.email.valid);
        assert_eq!(view.email.error.as_deref(), Some("Email cannot be empty"));

        assert_eq!(view.age.value, ""); // Option<u32> None maps to empty string
        assert!(view.age.valid); // Age (None) is initially valid (optional)
        assert!(view.age.error.is_none());

        assert_eq!(view.address.value, "");
        assert!(!view.address.valid);
        assert_eq!(
            view.address.error.as_deref(),
            Some("Address cannot be empty")
        );

        assert!(!view.submitted);
        assert!(view.is_editing_form); // Form starts in editing mode
        assert!(!view.can_submit); // Initially false due to invalid fields
        assert_eq!(view.status_message, "Please correct the errors.");
    }

    #[test]
    fn update_username_valid() {
        let app = App::default();
        let mut model = Model::default();

        let _ = app.update(
            Event::SetFieldEditing {
                // Assuming this event is still desired for UI hints
                ident: FieldIdent::Username,
                editing: true,
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "TestUser".to_string(),
            },
            &mut model,
            &(),
        );

        assert_eq!(model.form.username.value, "TestUser");
        assert!(model.form.username.valid);
        assert!(model.form.username.dirty);
        assert!(model.form.username.editing); // Assuming SetFieldEditing worked

        let view = app.view(&model);
        assert_eq!(view.username.value, "TestUser");
        assert!(view.username.valid);
        assert_eq!(view.status_message, "Form has unsaved changes");
    }

    #[test]
    fn update_username_invalid_empty() {
        let app = App::default();
        let mut model = Model::default();
        let _ = app.update(
            Event::SetFieldEditing {
                ident: FieldIdent::Username,
                editing: true,
            },
            &mut model,
            &(),
        );

        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "".to_string(),
            },
            &mut model,
            &(),
        );
        assert!(!model.form.username.valid);
        assert_eq!(
            model.form.username.error.as_deref(),
            Some("Username cannot be empty")
        );

        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "Te".to_string(),
            },
            &mut model,
            &(),
        );
        assert!(!model.form.username.valid);
        assert_eq!(
            model.form.username.error.as_deref(),
            Some("Username must be at least 3 characters")
        );
    }

    #[test]
    fn update_age_valid_and_invalid() {
        let app = App::default();
        let mut model = Model::default();
        let _ = app.update(
            Event::SetFieldEditing {
                ident: FieldIdent::Age,
                editing: true,
            },
            &mut model,
            &(),
        );

        // Valid age
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Age,
                value: "25".to_string(),
            },
            &mut model,
            &(),
        );
        assert!(model.form.age.valid);
        assert_eq!(model.form.age.value, Some(25));
        assert!(model.form.age.error.is_none());

        // Invalid age (text "abc" -> becomes None)
        // Validator for age treats None as Ok.
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Age,
                value: "abc".to_string(),
            },
            &mut model,
            &(),
        );
        assert!(model.form.age.valid);
        assert_eq!(model.form.age.value, None);
        assert!(model.form.age.error.is_none());

        // Invalid age (too young)
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Age,
                value: "17".to_string(),
            },
            &mut model,
            &(),
        );
        assert!(!model.form.age.valid);
        assert_eq!(model.form.age.value, Some(17));
        assert_eq!(
            model.form.age.error.as_deref(),
            Some("Age must be between 18 and 120")
        );

        // Empty age string "" -> becomes None
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Age,
                value: "".to_string(),
            },
            &mut model,
            &(),
        );
        assert!(model.form.age.valid);
        assert_eq!(model.form.age.value, None);
        assert!(model.form.age.error.is_none());
    }

    #[test]
    fn submit_empty_form_fails_validation() {
        let app = App::default();
        let mut model = Model::default(); // Initial validation from Form::default()

        let _ = app.update(Event::Submit, &mut model, &());

        assert!(!model.form.submitted);
        assert!(model.form.is_editing); // Stays true due to validation errors

        assert!(!model.form.username.valid);
        assert_eq!(
            model.form.username.error.as_deref(),
            Some("Username cannot be empty")
        );
        // ... (other fields as in initial_state due to Form::default())

        let view = app.view(&model);
        assert!(!view.can_submit);
        assert!(view.username.touched); // Submit touches all fields via form.touch_all()
                                        // ... (other fields touched)
    }

    #[test]
    fn submit_partially_filled_form_shows_errors() {
        let app = App::default();
        let mut model = Model::default();

        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "TestUser".to_string(),
            },
            &mut model,
            &(),
        );
        // Email is still empty and thus invalid from Form::default()

        let _ = app.update(Event::Submit, &mut model, &());

        assert!(!model.form.submitted);
        assert!(model.form.is_editing);
        assert!(model.form.username.valid);
        assert!(!model.form.email.valid);
        assert_eq!(
            model.form.email.error.as_deref(),
            Some("Email cannot be empty")
        );

        let view = app.view(&model);
        assert_eq!(view.email.error.as_deref(), Some("Email cannot be empty"));
        assert!(view.email.touched);
    }

    #[test]
    fn submit_valid_form() {
        let app = App::default();
        let mut model = Model::default();

        // Make all fields valid
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "ValidUser".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Email,
                value: "valid@example.com".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Age,
                value: "30".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Address,
                value: "123 Main St".to_string(),
            },
            &mut model,
            &(),
        );

        assert!(model.form.is_valid()); // Check all fields valid via Form method

        let _ = app.update(Event::Submit, &mut model, &());

        assert!(model.form.submitted);
        assert!(!model.form.is_editing); // Form set to not editing

        let view = app.view(&model);
        assert!(view.submitted);
        assert!(!view.is_editing_form);
        assert!(!view.username.editing); // Fields editing status updated by form.set_editing
        assert_eq!(view.status_message, "Form Submitted Successfully!");
        assert!(!view.can_submit);
    }

    #[test]
    fn edit_after_submit() {
        let app = App::default();
        let mut model = Model::default();

        // Make form valid and submit it
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "User".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Email,
                value: "user@example.com".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Age,
                value: "42".to_string(),
            },
            &mut model,
            &(),
        ); // "42" -> Some(42)
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Address,
                value: "Addr".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(Event::Submit, &mut model, &()); // Submit the form

        assert!(model.form.submitted);
        assert!(!model.form.is_editing);

        let _ = app.update(Event::Edit, &mut model, &()); // Edit the form

        assert!(!model.form.submitted); // Submitted flag reset
        assert!(model.form.is_editing); // Form is editable again

        let view = app.view(&model);
        assert!(view.is_editing_form);
        // Status message will depend on dirtiness. Since values are same as before submission,
        // and Field.reset doesn't occur on Edit, they might still be dirty relative to initial empty.
        // Or if Edit resets dirty flags (it doesn't by default in Form), it might be "Please fill...".
        // Current Form::set_editing does not reset dirty flags.
        // Fields were dirty before submit, they remain dirty.
        assert_eq!(view.status_message, "Form has unsaved changes");
    }

    #[test]
    fn reset_form_clears_fields_and_resets_state() {
        let app = App::default();
        let mut model = Model::default();

        // Change some values
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "Test".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Email,
                value: "test@example.com".to_string(),
            },
            &mut model,
            &(),
        );

        assert!(model.form.username.dirty);

        let _ = app.update(Event::ResetForm, &mut model, &());

        // Form::reset() re-initializes to Form::default()
        assert_eq!(model.form.username.value, "");
        assert!(!model.form.username.dirty); // Default field is not dirty
        assert!(!model.form.username.touched); // Default field is not touched
        assert!(!model.form.username.valid); // Default username "" is invalid
        assert_eq!(
            model.form.username.error.as_deref(),
            Some("Username cannot be empty")
        );

        assert_eq!(model.form.email.value, "");
        assert!(!model.form.email.valid);
        assert_eq!(
            model.form.email.error.as_deref(),
            Some("Email cannot be empty")
        );

        assert!(!model.form.submitted);
        assert!(model.form.is_editing); // Reset sets is_editing to true

        let view = app.view(&model);
        assert_eq!(view.username.value, "");
        assert_eq!(view.status_message, "Please correct the errors.");
    }

    #[test]
    fn touch_field_updates_touched_flag() {
        let app = App::default();
        let mut model = Model::default();

        assert!(!model.form.username.touched);
        let _ = app.update(
            Event::TouchField {
                ident: FieldIdent::Username,
            },
            &mut model,
            &(),
        );
        assert!(model.form.username.touched);
    }
    // Tests for SetFieldEditing might need adjustment based on how it's intended to interact
    // with the overall form's editing state (model.form.is_editing).
    // The current implementation of SetFieldEditing in `update` only affects individual field's `editing` flag.

    #[test]
    fn address_suggestions_are_cleared_when_editing_disabled() {
        let app = App::default();
        let mut model = Model::default();

        // Add some suggestions
        model.address_suggestions = vec![
            AddressSuggestion {
                street: "123 Main St".to_string(),
                city: "London".to_string(),
                postcode: "SW1A 1AA".to_string(),
                country: "UK".to_string(),
                combined: "123 Main St, London, SW1A 1AA, UK".to_string(),
            }
        ];

        // Disable editing for address field
        let _ = app.update(
            Event::SetFieldEditing {
                ident: FieldIdent::Address,
                editing: false,
            },
            &mut model,
            &(),
        );

        assert!(model.address_suggestions.is_empty());
    }

    #[test]
    fn address_suggestions_are_cleared_on_form_submit() {
        let app = App::default();
        let mut model = Model::default();

        // Add some suggestions
        model.address_suggestions = vec![
            AddressSuggestion {
                street: "123 Main St".to_string(),
                city: "London".to_string(),
                postcode: "SW1A 1AA".to_string(),
                country: "UK".to_string(),
                combined: "123 Main St, London, SW1A 1AA, UK".to_string(),
            }
        ];

        // Make form valid and submit
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Username,
                value: "ValidUser".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Email,
                value: "valid@example.com".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Age,
                value: "30".to_string(),
            },
            &mut model,
            &(),
        );
        let _ = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Address,
                value: "123 Main St".to_string(),
            },
            &mut model,
            &(),
        );

        let _ = app.update(Event::Submit, &mut model, &());

        assert!(model.address_suggestions.is_empty());
    }

    #[test]
    fn address_suggestions_are_cleared_on_form_reset() {
        let app = App::default();
        let mut model = Model::default();

        // Add some suggestions
        model.address_suggestions = vec![
            AddressSuggestion {
                street: "123 Main St".to_string(),
                city: "London".to_string(),
                postcode: "SW1A 1AA".to_string(),
                country: "UK".to_string(),
                combined: "123 Main St, London, SW1A 1AA, UK".to_string(),
            }
        ];

        let _ = app.update(Event::ResetForm, &mut model, &());

        assert!(model.address_suggestions.is_empty());
    }

    #[test]
    fn address_suggestions_are_cleared_on_form_edit() {
        let app = App::default();
        let mut model = Model::default();

        // Add some suggestions
        model.address_suggestions = vec![
            AddressSuggestion {
                street: "123 Main St".to_string(),
                city: "London".to_string(),
                postcode: "SW1A 1AA".to_string(),
                country: "UK".to_string(),
                combined: "123 Main St, London, SW1A 1AA, UK".to_string(),
            }
        ];

        let _ = app.update(Event::Edit, &mut model, &());

        assert!(model.address_suggestions.is_empty());
    }

    #[test]
    fn address_field_updates_trigger_suggestion_fetch() {
        let app = App::default();
        let mut model = Model::default();

        // Ensure we're in editing mode
        model.form.set_editing(true);

        // Update address field to trigger suggestion fetch
        let mut cmd = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Address,
                value: "test".to_string(),
            },
            &mut model,
            &(),
        );

        // Verify that the address field was updated
        assert_eq!(model.form.address.value, "test");

        // The command should contain an HTTP request
        let mut request = cmd.effects().next().unwrap().expect_http();
        assert_eq!(
            &request.operation,
            &HttpRequest::get(format!("{}?query={}", ADDRESS_API_URL, "test"))
                .build()
        );

        // Simulate a successful response from the API
        let suggestions = vec![
            AddressSuggestion {
                street: "123 Test St".to_string(),
                city: "London".to_string(),
                postcode: "SW1A 1AA".to_string(),
                country: "UK".to_string(),
                combined: "123 Test St, London, SW1A 1AA, UK".to_string(),
            }
        ];

        let response = HttpResponse::ok()
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&suggestions).unwrap())
            .build();

        // Resolve the request with our simulated response
        request.resolve(HttpResult::Ok(response)).unwrap();

        // This should generate an AddressSuggestionsReceived event
        let event = cmd.events().next().unwrap().clone();
        assert!(matches!(event, Event::AddressSuggestionsReceived(AddressSuggestionsResult::Success(_))));

        // Send the AddressSuggestionsReceived event back to the app
        let mut cmd = app.update(event.clone(), &mut model, &());

        // The app should ask to render
        assert!(matches!(cmd.effects().next().unwrap(), Effect::Render(_)));

        // Verify that the suggestions were updated in the model
        if let Event::AddressSuggestionsReceived(AddressSuggestionsResult::Success(suggestions)) = event {
            assert_eq!(model.address_suggestions, suggestions);
        } else {
            panic!("Expected AddressSuggestionsResult::Success");
        }
    }

    #[test]
    fn address_suggestions_handles_error_response() {
        let app = App::default();
        let mut model = Model::default();

        // Ensure we're in editing mode
        model.form.set_editing(true);

        // Update address field to trigger suggestion fetch
        let mut cmd = app.update(
            Event::UpdateValue {
                ident: FieldIdent::Address,
                value: "test".to_string(),
            },
            &mut model,
            &(),
        );

        // Get the HTTP request
        let mut request = cmd.effects().next().unwrap().expect_http();

        // Simulate an error response
        request.resolve(HttpResult::Err(HttpError::Timeout)).unwrap();

        // This should generate an AddressSuggestionsReceived event with an error
        let event = cmd.events().next().unwrap().clone();
        assert!(matches!(event, Event::AddressSuggestionsReceived(AddressSuggestionsResult::Error)));

        // Send the error event back to the app
        let mut cmd = app.update(event.clone(), &mut model, &());

        // The app should ask to render
        assert!(matches!(cmd.effects().next().unwrap(), Effect::Render(_)));

        // Verify that suggestions were cleared
        assert!(model.address_suggestions.is_empty());
    }

    // Add a test that explicitly uses all variants of AddressSuggestionsResult
    #[test]
    fn test_address_suggestions_result_variants() {
        // This test exists to help the type tracer see all variants
        let success = AddressSuggestionsResult::Success(vec![
            AddressSuggestion {
                street: "123 Test St".to_string(),
                city: "London".to_string(),
                postcode: "SW1A 1AA".to_string(),
                country: "UK".to_string(),
                combined: "123 Test St, London, SW1A 1AA, UK".to_string(),
            }
        ]);
        let error = AddressSuggestionsResult::Error;

        // Use both variants in a way that forces the type tracer to analyze them
        let _ = match success {
            AddressSuggestionsResult::Success(_) => true,
            AddressSuggestionsResult::Error => false,
        };
        let _ = match error {
            AddressSuggestionsResult::Success(_) => false,
            AddressSuggestionsResult::Error => true,
        };
    }
}
