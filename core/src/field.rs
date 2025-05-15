use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

// PartialEq and Eq removed because Box<dyn Fn(...)> doesn't implement them.
// Can be manually implemented later if needed, comparing all fields except the validator.
#[derive(Serialize, Deserialize)]
pub struct Field<T> {
    pub value: T,
    pub initial_value: T,
    pub touched: bool,
    pub dirty: bool,
    pub error: Option<String>,
    pub valid: bool,
    pub editing: bool,
    #[serde(skip)]
    validator: Option<Arc<dyn Fn(&T) -> Result<(), String> + Send + Sync>>,
}

// Manual Debug implementation
impl<T: Debug> std::fmt::Debug for Field<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("value", &self.value)
            .field("initial_value", &self.initial_value)
            .field("touched", &self.touched)
            .field("dirty", &self.dirty)
            .field("error", &self.error)
            .field("valid", &self.valid)
            .field("editing", &self.editing)
            .field(
                "validator",
                &self.validator.as_ref().map(|_| "Arc<dyn Fn(...)>"),
            )
            .finish()
    }
}

// Added 'static lifetime and Send + Sync for T due to Arc<dyn Fn(&T)... Send + Sync>
impl<T: PartialEq + Clone + Send + Sync + Debug + 'static> Field<T> {
    pub fn new(
        initial: T,
        validator: Option<Arc<dyn Fn(&T) -> Result<(), String> + Send + Sync>>,
    ) -> Self {
        let mut field = Self {
            value: initial.clone(),
            initial_value: initial,
            touched: false,
            dirty: false,
            error: None,
            valid: true,
            editing: false,
            validator,
        };
        field.validate(); // Initial validation
        field
    }

    pub fn update_value(&mut self, new_value: T) {
        self.value = new_value.clone();
        self.dirty = self.value != self.initial_value;
        self.touched = true;
        self.validate();
    }

    pub fn validate(&mut self) {
        if let Some(validator_fn) = &self.validator {
            match validator_fn(&self.value) {
                Ok(_) => {
                    self.valid = true;
                    self.error = None;
                }
                Err(msg) => {
                    self.valid = false;
                    self.error = Some(msg);
                }
            }
        } else {
            self.valid = true;
            self.error = None;
        }
    }

    pub fn set_editing(&mut self, editing: bool) {
        self.editing = editing;
        if editing {
            self.touched = true;
        }
    }

    pub fn touch(&mut self) {
        self.touched = true;
        // self.validate(); // Decide if validation should run on touch
    }

    pub fn reset(&mut self) {
        self.value = self.initial_value.clone();
        self.touched = false;
        self.dirty = false;
        self.editing = false;
        self.validate();
    }

    pub fn set_error(&mut self, error_message: Option<String>) {
        self.error = error_message;
        self.valid = self.error.is_none();
    }
}

// Specific constructor for String fields for convenience
impl Field<String> {
    pub fn new_string(
        initial: &str,
        validator: Option<Arc<dyn Fn(&String) -> Result<(), String> + Send + Sync>>,
    ) -> Self {
        Self::new(initial.to_string(), validator)
    }
}

// Specific constructor for Option<u32> fields
impl Field<Option<u32>> {
    pub fn new_option_u32(
        initial: Option<u32>,
        validator: Option<Arc<dyn Fn(&Option<u32>) -> Result<(), String> + Send + Sync>>,
    ) -> Self {
        Self::new(initial, validator)
    }
}
