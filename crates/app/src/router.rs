//! All app routes. Tool pages get added here as modules ship (see features/).

use dioxus::prelude::*;

use crate::pages::{AllTools, ComingSoon, Roadmap};
use crate::shell::Shell;

#[derive(Debug, Clone, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        AllTools {},
        #[route("/roadmap")]
        Roadmap {},
        #[route("/:..segments")]
        ComingSoon { segments: Vec<String> },
}
