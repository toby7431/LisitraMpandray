// Empêche l'ouverture d'une console Windows en mode release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    eglise_gestion_tauri_lib::run();
}
