use std::fs::read_to_string;

use eyre::Context;

pub fn get_norms() -> eyre::Result<String> {
    let path = "BBAI_NORMS.md";

    read_to_string(path).context("Reading BBAI_NORMS.md file")
}
