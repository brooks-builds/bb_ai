mod message;

use anathema::runtime::Builder;
use eyre::Result;
use message::Message;

pub fn register(builder: &mut Builder<()>) -> Result<()> {
    builder.prototype(
        "message",
        "templates/prototypes/message.aml",
        || Message,
        || (),
    )?;

    Ok(())
}
