mod app;
mod pages;
mod router;
mod components;
mod prototypes;

use anathema::{
    prelude::{Backend, Document, TuiBackend},
    runtime::Runtime,
};
use eyre::Result;
use crate::{app::{App, AppState}, pages::chat::{ChatPage, ChatPageState}, router::{Router, RouterState}};

pub fn run() -> Result<()> {
    let doc = Document::new("@index");

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()?;

    backend.finalize();

    let mut builder = Runtime::builder(doc, &backend);

    bb_anathema_components::register_all(&mut builder)?;
    builder.component("index", "templates/index.aml", App, AppState::default())?;
    builder.component("router", "templates/router.aml", Router, RouterState::default())?;
    builder.component("chat_page", "templates/pages/chat.aml", ChatPage, ChatPageState::default())?;
    components::register(&mut builder)?;
    prototypes::register(&mut builder)?;
        

    builder.finish(&mut backend, |runtime, backend| runtime.run(backend))?;

    Ok(())
}
