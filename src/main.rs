use anyhow::Result;

use instr::options;
use instr::run;

mod tests;

fn main() -> Result<()> {
    let options = options().run();

    run(options)
}
