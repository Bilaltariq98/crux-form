use crate::field::Field;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Form {
    pub username: Field<String>,
    pub email: Field<String>,
    pub age: Field<Option<u32>>,
    pub address: Field<String>,
    pub submitted: bool,
    pub is_editing: bool, // Renamed from is_editing_form
}

impl Default for Form {
    fn default() -> Self {
        let username_validator = Arc::new(|v: &String| {
            if v.trim().is_empty() {
                Err("Username cannot be empty".to_string())
            } else if v.len() < 3 {
                Err("Username must be at least 3 characters".to_string())
            } else {
                Ok(())
            }
        });

        let email_validator = Arc::new(|v: &String| {
            if v.trim().is_empty() {
                Err("Email cannot be empty".to_string())
            } else if !v.contains('@') || !v.contains('.') {
                Err("Invalid email format".to_string())
            } else {
                Ok(())
            }
        });

        let age_validator = Arc::new(|v_opt: &Option<u32>| {
            match v_opt {
                Some(age_val) if *age_val >= 18 && *age_val <= 120 => Ok(()),
                Some(_) => Err("Age must be between 18 and 120".to_string()),
                None => Ok(()), // Assuming None is acceptable (optional field or parse error treated as None)
            }
        });

        let address_validator = Arc::new(|v: &String| {
            if v.trim().is_empty() {
                Err("Address cannot be empty".to_string())
            } else {
                Ok(())
            }
        });

        Self {
            username: Field::new_string("", Some(username_validator)),
            email: Field::new_string("", Some(email_validator)),
            age: Field::new_option_u32(None, Some(age_validator)),
            address: Field::new_string("", Some(address_validator)),
            submitted: false,
            is_editing: true,
        }
    }
}

impl Form {
    pub fn new() -> Self {
        Self::default()
    }

    // Placeholder for validate_all
    pub fn validate_all(&mut self) {
        self.username.validate();
        self.email.validate();
        self.age.validate();
        self.address.validate();
    }

    // Placeholder for is_valid
    pub fn is_valid(&self) -> bool {
        self.username.valid && self.email.valid && self.age.valid && self.address.valid
    }

    // Placeholder for touch_all
    pub fn touch_all(&mut self) {
        self.username.touch();
        self.email.touch();
        self.age.touch();
        self.address.touch();
    }

    // Placeholder for reset
    pub fn reset(&mut self) {
        let default_form = Form::default();
        self.username = default_form.username;
        self.email = default_form.email;
        self.age = default_form.age;
        self.address = default_form.address;

        self.submitted = false;
        self.is_editing = true;
        // After reset, fields are re-validated internally by their reset methods.
    }

    // Placeholder for set_editing for the whole form
    pub fn set_editing(&mut self, editing: bool) {
        self.is_editing = editing;
        // Optionally, cascade this to individual fields' editing status if they have one
        self.username.set_editing(editing);
        self.email.set_editing(editing);
        self.age.set_editing(editing);
        self.address.set_editing(editing);
    }

    // You might also want a method to determine if the form can be submitted
    pub fn can_submit(&self) -> bool {
        self.is_valid() && self.is_editing
    }
}

// We need to ensure Field has new_string and new_option_u32 constructors.
// Based on previous context, Field::new is generic.
// Let's assume Field has appropriate constructors or we'll add them to field.rs if necessary.
// For now, I'll use the Field::new method similar to how it was used in Model::default,
// which implies `new_string` and `new_option_u32` might be helper methods on Field itself,
// or the types are constructed and then passed to a generic `Field::new`.
// The previous `Model::default()` used `Field::new_string` and `Field::new_option_u32`.
// I'll assume these exist in `crate::field`. If not, this will be a compile error to fix.
