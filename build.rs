#![allow(clippy::eq_op)]

use anyhow::Result;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

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

    let out_dir = env::var("OUT_DIR")?;
    let final_path = Path::new(&out_dir).join("tables.rs");
    let mut file = File::create(final_path)?;

    writeln!(
        file,
        r##"#![allow(dead_code)]
#[repr(usize)]
pub enum Tuning {{
    Pf_,
    Pf3,
    Pf5,
    Pf3_5,
    Pf7,
    Pf3_7,
    Pf5_7,
    Pf3_5_7,
    Pf11,
    Pf3_11,
    Pf5_11,
    Pf3_5_11,
    Pf7_11,
    Pf3_7_11,
    Pf5_7_11,
    Pf3_5_7_11,
    Pf13,
    Pf3_13,
    Pf5_13,
    Pf3_5_13,
    Pf7_13,
    Pf3_7_13,
    Pf5_7_13,
    Pf3_5_7_13,
    Pf11_13,
    Pf3_11_13,
    Pf5_11_13,
    Pf3_5_11_13,
    Pf7_11_13,
    Pf3_7_11_13,
    Pf5_7_11_13,
    Pf3_5_7_11_13,
    Pf17,
    Pf17_3,
    Pf17_5,
    Pf17_3_5,
    Pf17_7,
    Pf17_3_7,
    Pf17_5_7,
    Pf17_3_5_7,
    Pf17_11,
    Pf17_3_11,
    Pf17_5_11,
    Pf17_3_5_11,
    Pf17_7_11,
    Pf17_3_7_11,
    Pf17_5_7_11,
    Pf17_3_5_7_11,
    Pf17_13,
    Pf17_3_13,
    Pf17_5_13,
    Pf17_3_5_13,
    Pf17_7_13,
    Pf17_3_7_13,
    Pf17_5_7_13,
    Pf17_3_5_7_13,
    Pf17_11_13,
    Pf17_3_11_13,
    Pf17_5_11_13,
    Pf17_3_5_11_13,
    Pf17_7_11_13,
    Pf17_3_7_11_13,
    Pf17_5_7_11_13,
    Pf17_3_5_7_11_13,
}}

pub const PYTHAGOREAN: Tuning = Tuning::Pf3;
pub const FIVE_LIMIT: Tuning = Tuning::Pf3_5;
pub const SEVEN_LIMIT: Tuning = Tuning::Pf3_5_7;
pub const ELEVEN_LIMIT: Tuning = Tuning::Pf3_5_7_11;
pub const THIRTEEN_LIMIT: Tuning = Tuning::Pf3_5_7_11_13;
pub const SEVENTEEN_LIMIT: Tuning = Tuning::Pf17_3_5_7_11_13;"##
    )?;

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
