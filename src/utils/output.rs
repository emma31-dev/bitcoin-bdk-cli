use std::io::Write;

use crate::{commands::OutputFormatType, error::BDKCliError as Error};
use cli_table::{Table, print_stdout};
use serde::Serialize;

/// A trait for types that can be presented to the user.
pub trait FormatOutput: Serialize + Table {
    /// Formats the output according to the requested [`OutputFormatType`].
    fn format(&self, format: OutputFormatType) -> Result<String, Error> {
        match format {
            OutputFormatType::Json => serde_json::to_string_pretty(self)
                .map_err(|e| Error::Generic(format!("JSON serialization failed: {e}"))),
            OutputFormatType::Toml => toml::to_string_pretty(self)
                .map_err(|e| Error::Generic(format!("TOML serialization failed: {e}"))),
            OutputFormatType::Table => Ok("".into()),
        }
    }


    fn write_out<W: Write>(&self, mut writer: W, format: OutputFormatType) -> Result<(), Error> {
        match format {
            OutputFormatType::Table => {
                print_stdout(vec![self.clone()].table()?);
            }
            _ => {
                let output = self.format(format)?;
                writeln!(writer, "{}", output)
                    .map_err(|e| Error::Generic(format!("Failed to write output: {e}")))?;
            }
        }

        Ok(())
    }
}

impl<T: Serialize + Table> FormatOutput for T {}

/// A generic wrapper for commands that return a list of items.
#[derive(Serialize)]
pub struct ListResult<T> {
    pub count: usize,
    pub items: Vec<T>,
}

impl<T> ListResult<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            count: items.len(),
            items,
        }
    }
}
