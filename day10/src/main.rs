use std::io::BufRead;

use color_eyre::eyre::Result;
use common_utils::get_buffered_input;
fn main() -> Result<()> {
    color_eyre::install()?;
    get_buffered_input();
    Ok(())
}
