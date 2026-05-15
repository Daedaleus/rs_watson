pub mod csv;

use std::io::Write;

use rs_watson::Frame;

pub trait Exporter {
    type Error: std::error::Error;

    fn export<W: Write>(&self, frames: &[Frame], writer: W) -> Result<(), Self::Error>;
}
