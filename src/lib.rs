mod app;
mod components;
mod models;
mod pages;
mod services;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    // Affichage des panics Rust dans la console du navigateur
    console_error_panic_hook::set_once();
    // Montage du composant racine dans <body>
    leptos::mount::mount_to_body(app::App);
}
