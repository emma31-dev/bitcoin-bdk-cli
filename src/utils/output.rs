use std::io::Write;

use crate::{commands::OutputFormatType, error::BDKCliError as Error};
use serde::Serialize;
use cli_table::{format::Justify, Cell, Style, Table};

/// A trait for types that can be presented to the user.
pub trait FormatOutput: Serialize {
    /// Formats the output according to the requested [`OutputFormatType`].
    fn format(&self, format: OutputFormatType) -> Result<String, Error> {
        match format {
            OutputFormatType::Json => serde_json::to_string_pretty(self)
                .map_err(|e| Error::Generic(format!("JSON serialization failed: {e}"))),
            OutputFormatType::Toml => toml::to_string_pretty(self)
                .map_err(|e| Error::Generic(format!("TOML serialization failed: {e}"))),
            OutputFormatType::Table => self.format_table(),
        }
    }

    /// Renders the output as a table.
    ///
    /// The value is serialized to a JSON array of records, converted into rows
    /// of strings, and rendered with [`cli_table`]. When the value is not an
    /// array (e.g. a single object or scalar), it is treated as a one-row table.
    /// The default implementation falls back to JSON when the value cannot be
    /// represented as rows.
    fn format_table(&self) -> Result<String, Error> {

        // Serialize the value into a generic JSON representation so we can
        // inspect its shape regardless of the concrete type.
        let value = serde_json::to_value(self)
            .map_err(|e| Error::Generic(format!("Table serialization failed: {e}")))?;

        // Normalize into a list of rows. A top-level array is treated as the
        // list of rows; anything else is treated as a single row.
        let rows: Vec<serde_json::Value> = match value {
            serde_json::Value::Array(items) => items,
            other => vec![other],
        };

        // Collect the ordered set of column names from all row objects.
        let mut columns: Vec<String> = Vec::new();
        for row in &rows {
            if let serde_json::Value::Object(map) = row {
                for key in map.keys() {
                    if !columns.iter().any(|c| c == key) {
                        columns.push(key.clone());
                    }
                }
            }
        }

        if columns.is_empty() {
            // Not a set of records; fall back to JSON.
            return serde_json::to_string_pretty(self)
                .map_err(|e| Error::Generic(format!("Table serialization failed: {e}")));
        }

        // Build the header row.
        let header: Vec<_> = columns
            .iter()
            .map(|name| name.as_str().cell().bold(true).justify(Justify::Left))
            .collect();

        // Build the body rows as string cells.
        let table_rows: Vec<Vec<_>> = rows
            .iter()
            .map(|row| {
                columns
                    .iter()
                    .map(|column| {
                        let text = match row.get(column) {
                            Some(serde_json::Value::String(s)) => s.clone(),
                            Some(other) => other.to_string(),
                            None => String::new(),
                        };
                        text.cell()
                    })
                    .collect()
            })
            .collect();

        let table = table_rows
            .table()
            .title(header)
            .display()
            .map_err(|e| Error::Generic(format!("Table rendering failed: {e}")))?;

        Ok(table.to_string())
    }


    fn write_out<W: Write>(
        &self,
        mut writer: W,
        format: OutputFormatType,
    ) -> Result<(), Error> {
        let output = self.format(format)?;

        writeln!(writer, "{}", output)
            .map_err(|e| Error::Generic(format!("Failed to write output: {e}")))
    }
}

impl<T: Serialize> FormatOutput for T {}

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
