//! All app routes. Tool pages are dispatched by id in `pages::ToolPage`.

use dioxus::prelude::*;

use crate::pages::{AllTools, ComingSoon, ModulePage, Roadmap, ToolPage};
use crate::shell::Shell;

#[derive(Debug, Clone, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        AllTools {},
        #[route("/roadmap")]
        Roadmap {},
        // `key` is reserved by Dioxus, so the segment is named `module`.
        #[route("/module/:module", ModulePage)]
        Module { module: String },
        #[route("/tool/:id", ToolPage)]
        Tool { id: String },
        #[route("/:..segments")]
        ComingSoon { segments: Vec<String> },
}
