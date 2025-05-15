pub mod capabilities;
pub mod field;
pub mod form;
pub mod app;

use lazy_static::lazy_static;

pub use crux_core::bridge::{Bridge, Request};
pub use crux_core::{Core, ResolveError};
pub use crux_http as http;

pub use app::*;
// We are not using sse capability for the form app for now
// pub use capabilities::sse;

// TODO hide this plumbing

uniffi::include_scaffolding!("form"); // Changed from "shared"

lazy_static! {
    // Changed App to FormApp
    static ref CORE: Bridge<App> = Bridge::new(Core::new());
}

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn process_event(data: &[u8]) -> Vec<u8> {
    match CORE.process_event(data) {
        Ok(effects) => effects,
        Err(e) => panic!("{e}"),
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn handle_response(id: u32, data: &[u8]) -> Vec<u8> {
    match CORE.handle_response(id, data) {
        Ok(effects) => effects,
        Err(e) => panic!("{e}"),
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn view() -> Vec<u8> {
    match CORE.view() {
        Ok(view) => view,
        Err(e) => panic!("{e}"),
    }
}
