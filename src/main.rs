mod app;
mod simulator;

use leptos::*;
use crate::app::App;

fn main() {
    // Enable detailed Rust panic stacks in browser console
    console_error_panic_hook::set_once();
    
    // Initialize browser logging
    _ = console_log::init_with_level(log::Level::Debug);

    // Mount the main App component
    mount_to_body(|| view! { <App /> });
}
