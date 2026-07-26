mod app;
mod api;
mod pages;
mod utils;

use app::App;
use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}
