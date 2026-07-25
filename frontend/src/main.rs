mod app;
mod api;

use app::App;
use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}
