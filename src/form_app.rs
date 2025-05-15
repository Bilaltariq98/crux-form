use crux_core::{
    macros::effect,
    render::{render, RenderOperation},
    Command,
};
// We might need Http capability later for submitting the form
// use crux_http::{command::Http, protocol::HttpRequest};
use serde::{Deserialize, Serialize};
// Removed Arc import as it's now encapsulated in Form
// Removed Field import as it's now encapsulated in Form

use crate::form::Form; // Import the new Form struct
use crate::field::Field; // Still need this for FieldViewModel mapping for now

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
    pub form: Form, // Model now contains an instance of Form
}

impl Default for Model {
    fn default() -> Self {
        Self {
            form: Form::default(), // Initialize Form using its Default impl
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
    pub submitted: bool,
    pub is_editing_form: bool, // This maps to form.is_editing
    pub status_message: String,
    pub can_submit: bool, // This maps to form.can_submit()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    UpdateValue { ident: FieldIdent, value: String },
    TouchField { ident: FieldIdent },
    SetFieldEditing { ident: FieldIdent, editing: bool }, // This might now apply to the whole form or specific fields if Form manages that
    Submit,
    Edit, // Allows re-editing the form after submission
    ResetForm,
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    // We might add other effects later, e.g., for HTTP requests on submit
}

#[derive(Default)]
pub struct FormApp;

impl crux_core::App for FormApp {
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
                    FieldIdent::Address => model.form.address.update_value(value),
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
                    FieldIdent::Address => model.form.address.touch(),
                }
                render()
            }
            Event::SetFieldEditing { ident, editing } => {
                // This event might need re-evaluation.
                // Does it set the whole form's editing state or a specific field?
                // The Form struct has `set_editing` for the whole form.
                // Individual fields also have `set_editing`.
                // For now, let's assume it's for a specific field, consistent with FieldIdent.
                if !model.form.is_editing && editing { // If form is not editable, cannot start editing a field.
                    return Command::done();
                }
                match ident {
                    FieldIdent::Username => model.form.username.set_editing(editing),
                    FieldIdent::Email => model.form.email.set_editing(editing),
                    FieldIdent::Age => model.form.age.set_editing(editing),
                    FieldIdent::Address => model.form.address.set_editing(editing),
                }
                // If *any* field starts editing, ensure the main form is_editing flag is true.
                // However, the Form's set_editing(true) might be better controlled by an `Edit` event.
                // This part depends on desired UX. For now, setting field editing implies form might be in an editing phase.
                // If `editing` is false for all fields, should `model.form.is_editing` become false?
                // This logic is getting complex here and might be better managed by `Form::set_editing`.
                // A simpler approach: an `Edit` event makes the whole form editable.
                // `Submit` makes it non-editable. `SetFieldEditing` just toggles a field's focus/UI state.
                render()
            }
            Event::Submit => {
                if !model.form.is_editing {
                    return Command::done();
                }

                model.form.touch_all(); // Mark all fields as touched
                model.form.validate_all(); // Validate all fields

                if model.form.is_valid() {
                    model.form.submitted = true;
                    model.form.set_editing(false); // Form is no longer editable
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
                render()
            }
            Event::ResetForm => {
                model.form.reset(); // Resets form to default, sets is_editing to true
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

        let map_option_u32_field_to_view_model =
            |field: &Field<Option<u32>>| FieldViewModel {
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
            submitted: model.form.submitted,
            is_editing_form: model.form.is_editing,
            can_submit: model.form.can_submit(),
            status_message,
        }
    }
}

// The Model::new() method is no longer needed as Default is sufficient and simpler.
// If specific initialization beyond Form::default() was needed for Model,
// Model::new() could be:
// impl Model {
//     pub fn new() -> Self {
//         Self { form: Form::new() /* or custom Form setup */ }
//     }
// }


#[cfg(test)]
mod tests {
    use super::{Event, FieldIdent, FormApp, Model};
    use crux_core::{App as _};

    #[test]
    fn initial_state() {
        let app = FormApp::default();
        let model = Model::default(); // Model::default() now initializes form with its defaults
        let view = app.view(&model);

        assert_eq!(view.username.value, "");
        assert!(!view.username.valid);
        assert_eq!(view.username.error.as_deref(), Some("Username cannot be empty"));
        
        assert_eq!(view.email.value, "");
        assert!(!view.email.valid);
        assert_eq!(view.email.error.as_deref(), Some("Email cannot be empty"));

        assert_eq!(view.age.value, ""); // Option<u32> None maps to empty string
        assert!(view.age.valid); // Age (None) is initially valid (optional)
        assert!(view.age.error.is_none());

        assert_eq!(view.address.value, "");
        assert!(!view.address.valid);
        assert_eq!(view.address.error.as_deref(), Some("Address cannot be empty"));
        
        assert!(!view.submitted);
        assert!(view.is_editing_form); // Form starts in editing mode
        assert!(!view.can_submit); // Initially false due to invalid fields
        assert_eq!(view.status_message, "Please correct the errors.");
    }

    #[test]
    fn update_username_valid() {
        let app = FormApp::default();
        let mut model = Model::default();

        let _ = app.update(
            Event::SetFieldEditing { // Assuming this event is still desired for UI hints
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
        let app = FormApp::default();
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
        let app = FormApp::default();
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
        let app = FormApp::default();
        let mut model = Model::default(); // Initial validation from Form::default()

        let _ = app.update(Event::Submit, &mut model, &());

        assert!(!model.form.submitted);
        assert!(model.form.is_editing); // Stays true due to validation errors

        assert!(!model.form.username.valid);
        assert_eq!(model.form.username.error.as_deref(), Some("Username cannot be empty"));
        // ... (other fields as in initial_state due to Form::default())
        
        let view = app.view(&model);
        assert!(!view.can_submit);
        assert!(view.username.touched); // Submit touches all fields via form.touch_all()
        // ... (other fields touched)
    }

    #[test]
    fn submit_partially_filled_form_shows_errors() {
        let app = FormApp::default();
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
        assert_eq!(model.form.email.error.as_deref(), Some("Email cannot be empty"));

        let view = app.view(&model);
        assert_eq!(view.email.error.as_deref(), Some("Email cannot be empty"));
        assert!(view.email.touched);
    }

    #[test]
    fn submit_valid_form() {
        let app = FormApp::default();
        let mut model = Model::default();

        // Make all fields valid
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Username, value: "ValidUser".to_string(), }, &mut model, &(),);
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Email, value: "valid@example.com".to_string(), }, &mut model, &(),);
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Age, value: "30".to_string(), }, &mut model, &(), );
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Address, value: "123 Main St".to_string(), }, &mut model, &(),);

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
        let app = FormApp::default();
        let mut model = Model::default();
        
        // Make form valid and submit it
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Username, value: "User".to_string(), }, &mut model, &(),);
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Email, value: "user@example.com".to_string(), }, &mut model, &(),);
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Age, value: "42".to_string(), }, &mut model, &(),); // "42" -> Some(42)
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Address, value: "Addr".to_string(), }, &mut model, &(),);
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
        let app = FormApp::default();
        let mut model = Model::default();

        // Change some values
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Username, value: "Test".to_string(), }, &mut model, &(), );
        let _ = app.update( Event::UpdateValue { ident: FieldIdent::Email, value: "test@example.com".to_string(), }, &mut model, &(),);
        
        assert!(model.form.username.dirty);

        let _ = app.update(Event::ResetForm, &mut model, &());

        // Form::reset() re-initializes to Form::default()
        assert_eq!(model.form.username.value, "");
        assert!(!model.form.username.dirty); // Default field is not dirty
        assert!(!model.form.username.touched); // Default field is not touched
        assert!(!model.form.username.valid); // Default username "" is invalid
        assert_eq!(model.form.username.error.as_deref(), Some("Username cannot be empty"));
        
        assert_eq!(model.form.email.value, "");
        assert!(!model.form.email.valid);
        assert_eq!(model.form.email.error.as_deref(), Some("Email cannot be empty"));

        assert!(!model.form.submitted);
        assert!(model.form.is_editing); // Reset sets is_editing to true

        let view = app.view(&model);
        assert_eq!(view.username.value, "");
        assert_eq!(view.status_message, "Please correct the errors.");
    }

    #[test]
    fn touch_field_updates_touched_flag() {
        let app = FormApp::default();
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
}
