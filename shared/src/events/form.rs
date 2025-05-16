use crux_core::{render::render, Command};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

use crate::app::{Effect, Event};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum FieldIdent {
    Username,
    Email,
    Age,
    Address,
}

pub trait Validatable {
    fn is_valid(&self) -> bool;
    fn error_message(&self) -> Option<String>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Field<T: Clone + PartialEq + Validatable> {
    pub value: T,
    pub initial_value: T,
    pub touched: bool,
    pub dirty: bool,
    pub error: Option<String>,
    pub valid: bool,
    pub editing: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Username(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Email(pub String);

impl From<&str> for Username {
    fn from(s: &str) -> Self {
        Username(s.to_string())
    }
}

impl From<&str> for Email {
    fn from(s: &str) -> Self {
        Email(s.to_string())
    }
}

impl Validatable for Username {
    fn is_valid(&self) -> bool {
        self.0.len() >= 3
    }

    fn error_message(&self) -> Option<String> {
        if self.0.is_empty() {
            Some("Username cannot be empty".to_string())
        } else if self.0.len() < 3 {
            Some("Username must be at least 3 characters".to_string())
        } else {
            None
        }
    }
}

impl Validatable for Email {
    fn is_valid(&self) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        !self.0.is_empty() && email_regex.is_match(&self.0)
    }

    fn error_message(&self) -> Option<String> {
        if self.0.is_empty() {
            Some("Email cannot be empty".to_string())
        } else if !self.is_valid() {
            Some("Please enter a valid email address (e.g. user@example.com)".to_string())
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Form {
    pub username: Field<Username>,
    pub email: Field<Email>,
    pub age: Field<Option<u32>>,
    pub address: Field<String>,
    pub submitted: bool,
    pub is_editing: bool,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            username: Field {
                value: Username(String::new()),
                initial_value: Username(String::new()),
                touched: false,
                dirty: false,
                error: Some("Username cannot be empty".to_string()),
                valid: false,
                editing: false,
            },
            email: Field {
                value: Email(String::new()),
                initial_value: Email(String::new()),
                touched: false,
                dirty: false,
                error: Some("Email cannot be empty".to_string()),
                valid: false,
                editing: false,
            },
            age: Field {
                value: None,
                initial_value: None,
                touched: false,
                dirty: false,
                error: None,
                valid: true,
                editing: false,
            },
            address: Field {
                value: String::new(),
                initial_value: String::new(),
                touched: false,
                dirty: false,
                error: Some("Address cannot be empty".to_string()),
                valid: false,
                editing: false,
            },
            submitted: false,
            is_editing: true,
        }
    }
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FormViewModel {
    pub username: FieldViewModel,
    pub email: FieldViewModel,
    pub age: FieldViewModel,
    pub address: FieldViewModel,
    pub submitted: bool,
    pub is_editing_form: bool,
    pub status_message: String,
    pub can_submit: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FormEvent {
    UpdateValue { ident: FieldIdent, value: String },
    TouchField { ident: FieldIdent },
    SetFieldEditing { ident: FieldIdent, editing: bool },
    Submit,
    Edit,
    ResetForm,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FormHandler {
    form: Form,
}

impl FormHandler {
    pub fn new() -> Self {
        Self {
            form: Form::default(),
        }
    }

    pub fn handle_update_value(
        &mut self,
        ident: FieldIdent,
        value: String,
    ) -> Command<Effect, Event> {
        if !self.form.is_editing {
            return Command::done();
        }

        match ident {
            FieldIdent::Username => {
                self.form.username.set_value(value.as_str().into());
            }
            FieldIdent::Email => {
                self.form.email.set_value(value.as_str().into());
            }
            FieldIdent::Age => {
                let age = value.parse::<u32>().ok();
                self.form.age.set_value(age);
            }
            FieldIdent::Address => {
                self.form.address.set_value(value.clone());
                return Command::event(Event::FetchSuggestions { query: value }).then(render());
            }
        }

        self.form.validate_all();
        render()
    }

    pub fn handle_touch_field(&mut self, ident: FieldIdent) -> Command<Effect, Event> {
        if !self.form.is_editing {
            return Command::done();
        }
        match ident {
            FieldIdent::Username => self.form.username.mark_touched(),
            FieldIdent::Email => self.form.email.mark_touched(),
            FieldIdent::Age => self.form.age.mark_touched(),
            FieldIdent::Address => self.form.address.mark_touched(),
        }
        render()
    }

    pub fn handle_set_field_editing(
        &mut self,
        ident: FieldIdent,
        editing: bool,
    ) -> Command<Effect, Event> {
        if !self.form.is_editing && editing {
            return Command::done();
        }
        match ident {
            FieldIdent::Username => self.form.username.set_editing(editing),
            FieldIdent::Email => self.form.email.set_editing(editing),
            FieldIdent::Age => self.form.age.set_editing(editing),
            FieldIdent::Address => self.form.address.set_editing(editing),
        }
        render()
    }

    pub fn handle_submit(&mut self) -> Command<Effect, Event> {
        self.form.touch_all();
        self.form.validate_all();

        if self.form.is_valid() {
            self.form.submitted = true;
            self.form.set_editing(false);
            return Command::event(Event::ClearSuggestions).then(render());
        } else {
            self.form.submitted = false;
        }
        render()
    }

    pub fn handle_edit(&mut self) -> Command<Effect, Event> {
        self.form.submitted = false;
        self.form.set_editing(true);
        render()
    }

    pub fn handle_reset(&mut self) -> Command<Effect, Event> {
        self.form.reset();
        Command::event(Event::ClearSuggestions).then(render())
    }

    pub fn get_form(&self) -> &Form {
        &self.form
    }

    pub fn view(&self) -> FormViewModel {
        let username_vm = FieldViewModel {
            value: self.form.username.value.0.clone(),
            initial_value: self.form.username.initial_value.0.clone(),
            touched: self.form.username.touched,
            dirty: self.form.username.dirty,
            error: self.form.username.error.clone(),
            valid: self.form.username.valid,
            editing: self.form.username.editing,
        };

        let email_vm = FieldViewModel {
            value: self.form.email.value.0.clone(),
            initial_value: self.form.email.initial_value.0.clone(),
            touched: self.form.email.touched,
            dirty: self.form.email.dirty,
            error: self.form.email.error.clone(),
            valid: self.form.email.valid,
            editing: self.form.email.editing,
        };

        let age_vm = FieldViewModel {
            value: self
                .form
                .age
                .value
                .map_or_else(String::new, |v| v.to_string()),
            initial_value: self
                .form
                .age
                .initial_value
                .map_or_else(String::new, |v| v.to_string()),
            touched: self.form.age.touched,
            dirty: self.form.age.dirty,
            error: self.form.age.error.clone(),
            valid: self.form.age.valid,
            editing: self.form.age.editing,
        };

        let address_vm = FieldViewModel {
            value: self.form.address.value.clone(),
            initial_value: self.form.address.initial_value.clone(),
            touched: self.form.address.touched,
            dirty: self.form.address.dirty,
            error: self.form.address.error.clone(),
            valid: self.form.address.valid,
            editing: self.form.address.editing,
        };

        FormViewModel {
            username: username_vm,
            email: email_vm,
            age: age_vm,
            address: address_vm,
            submitted: self.form.submitted,
            is_editing_form: self.form.is_editing,
            status_message: if self.form.submitted {
                "Form Submitted Successfully!".to_string()
            } else if !self.form.is_editing {
                "Form data (View only)".to_string()
            } else if self.form.username.dirty
                || self.form.email.dirty
                || self.form.age.dirty
                || self.form.address.dirty
            {
                "Form has unsaved changes".to_string()
            } else if !self.form.is_valid() {
                "Please correct the errors.".to_string()
            } else {
                "Please fill out the form.".to_string()
            },
            can_submit: self.form.can_submit(),
        }
    }
}

impl<T: Clone + PartialEq + Validatable> Field<T> {
    pub fn set_value(&mut self, value: T) {
        self.value = value;
        self.dirty = self.value != self.initial_value;
        self.validate();
    }

    pub fn mark_touched(&mut self) {
        self.touched = true;
        self.validate();
    }

    pub fn set_editing(&mut self, editing: bool) {
        self.editing = editing;
    }

    fn validate(&mut self) {
        self.valid = self.value.is_valid();
        self.error = self.value.error_message();
    }
}

impl Validatable for String {
    fn is_valid(&self) -> bool {
        !self.is_empty()
    }

    fn error_message(&self) -> Option<String> {
        if self.is_empty() {
            Some("Field cannot be empty".to_string())
        } else {
            None
        }
    }
}

impl Validatable for Option<u32> {
    fn is_valid(&self) -> bool {
        match self {
            Some(age) => *age >= 18 && *age <= 120,
            None => true,
        }
    }

    fn error_message(&self) -> Option<String> {
        match self {
            Some(age) if *age < 18 || *age > 120 => {
                Some("Age must be between 18 and 120".to_string())
            }
            _ => None,
        }
    }
}

impl Form {
    pub fn touch_all(&mut self) {
        self.username.mark_touched();
        self.email.mark_touched();
        self.age.mark_touched();
        self.address.mark_touched();
    }

    pub fn validate_all(&mut self) {
        self.username.validate();
        self.email.validate();
        self.age.validate();
        self.address.validate();
    }

    pub fn is_valid(&self) -> bool {
        self.username.valid && self.email.valid && self.age.valid && self.address.valid
    }

    pub fn set_editing(&mut self, editing: bool) {
        self.is_editing = editing;
        self.username.set_editing(editing);
        self.email.set_editing(editing);
        self.age.set_editing(editing);
        self.address.set_editing(editing);
    }

    pub fn reset(&mut self) {
        *self = Form::default();
    }

    pub fn can_submit(&self) -> bool {
        self.is_editing && self.is_valid()
    }
}

pub trait ToFieldViewModel {
    fn to_field_view_model(&self) -> FieldViewModel;
}

impl<T: ToString + Clone + PartialEq + Validatable> ToFieldViewModel for Field<T> {
    fn to_field_view_model(&self) -> FieldViewModel {
        FieldViewModel {
            value: self.value.to_string(),
            initial_value: self.initial_value.to_string(),
            touched: self.touched,
            dirty: self.dirty,
            error: self.error.clone(),
            valid: self.valid,
            editing: self.editing,
        }
    }
}

impl ToString for Username {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl ToString for Email {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone, Debug)]
pub struct UsernameString(pub Username);

#[derive(Clone, Debug)]
pub struct EmailString(pub Email);

#[derive(Clone, Debug)]
pub struct AgeString(pub Option<u32>);

impl ToString for UsernameString {
    fn to_string(&self) -> String {
        self.0 .0.clone()
    }
}

impl ToString for EmailString {
    fn to_string(&self) -> String {
        self.0 .0.clone()
    }
}

impl ToString for AgeString {
    fn to_string(&self) -> String {
        self.0.map_or_else(String::new, |v| v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Effect;

    #[test]
    fn test_form_handler_update_value() {
        let mut handler = FormHandler::new();
        let mut cmd = handler.handle_update_value(FieldIdent::Username, "TestUser".to_string());
        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Render(_)));
        assert_eq!(handler.get_form().username.value.0, "TestUser");
    }

    #[test]
    fn test_form_handler_submit() {
        let mut handler = FormHandler::new();
        let mut cmd = handler.handle_submit();
        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Render(_)));
        assert!(!handler.get_form().submitted);
    }

    #[test]
    fn test_form_handler_reset() {
        let mut handler = FormHandler::new();
        handler.handle_update_value(FieldIdent::Username, "TestUser".to_string());
        let mut cmd = handler.handle_reset();
        let effect = cmd.effects().next().unwrap();
        assert!(matches!(effect, Effect::Render(_)));
        assert_eq!(handler.get_form().username.value.0, "");
    }
}
