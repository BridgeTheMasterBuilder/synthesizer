#![allow(clippy::eq_op)]

use std::fs::File;
use std::io::Write;

use anyhow::Result;

use crate::tables::TABLES;

mod tables;

fn emit_tables() -> Result<()> {
    fn emit_table(file: &mut File, table: &[f64; 12]) -> Result<()> {
        writeln!(file, "[")?;

        for octave in 0..8 {
            for interval in table {
                writeln!(file, "\t{}f64,", interval * f64::powi(2.0, octave))?;
            }
        }

        writeln!(file, "]")?;

        Ok(())
    }

    let mut file = File::create("src/tables.rs")?;

    writeln!(
        file,
        "pub const TABLES: [[f64; 12*8]; {}] = [",
        TABLES.len()
    )?;

    for table in TABLES {
        emit_table(&mut file, &table)?;
        writeln!(file, ",")?;
    }

    writeln!(file, "];")?;

    Ok(())
}

fn main() -> Result<()> {
    emit_tables()?;

    Ok(())
}
