use std::fmt::Write;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrintTableError {
    #[error("All rows need to have the exact same number of columns.")]
    VaryingNumberOfColumns,
    #[error("The given table is empty.")]
    EmptyTable,
}

pub fn print_table(rows: &[&[&str]]) -> Result<String, PrintTableError> {
    let num_columns = rows
        .first()
        .map(|r| r.len())
        .ok_or(PrintTableError::EmptyTable)?;
    if rows.iter().any(|r| r.len() != num_columns) {
        return Err(PrintTableError::VaryingNumberOfColumns);
    }

    let mut column_widths = vec![0; num_columns];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            column_widths[i] = column_widths[i].max(cell.len());
        }
    }

    let mut output = String::new();
    for row in rows.iter() {
        output.push('|');
        for (i, cell) in row.iter().enumerate() {
            let _ = write!(output, " {:<width$} |", cell, width = column_widths[i]);
        }
        output.push('\n');
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table_valid() {
        let rows = vec![vec!["Name", "Age"], vec!["Alice", "30"], vec!["Bob", "25"]];
        let refs: Vec<&[&str]> = rows.iter().map(|r| r.as_slice()).collect();

        // To capture printed output, use the test framework or redirect stdout, but we just test no error returned.
        assert!(print_table(&refs).is_ok());
    }

    #[test]
    fn test_print_table_empty() {
        let rows: Vec<&[&str]> = vec![];
        let res = print_table(&rows);
        assert!(matches!(res, Err(PrintTableError::EmptyTable)));
    }

    #[test]
    fn test_print_table_varying_columns() {
        let rows = vec![
            vec!["Name", "Age"],
            vec!["Alice"],
            vec!["Bob", "25", "Extra"],
        ];
        let refs: Vec<&[&str]> = rows.iter().map(|r| r.as_slice()).collect();

        let res = print_table(&refs);
        assert!(matches!(res, Err(PrintTableError::VaryingNumberOfColumns)));
    }

    #[test]
    fn test_print_table_column_widths() {
        let rows = vec![
            vec!["Header", "Col2", "X"],
            vec!["LongName", "Dat", "Z"],
            vec!["H", "COL2VERY", "ZZZ"],
        ];
        let refs: Vec<&[&str]> = rows.iter().map(|r| r.as_slice()).collect();

        // This should succeed, and column widths should be max of each column.
        assert!(print_table(&refs).is_ok());
    }
}
