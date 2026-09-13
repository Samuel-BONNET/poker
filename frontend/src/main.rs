mod app;
mod api;
mod components;
mod pages;
mod utils;

use app::App;

fn main() {
    leptos::mount::mount_to_body(App);
}
