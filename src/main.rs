use anyhow::Result;

use instr::options;
use instr::run;

fn main() -> Result<()> {
    let options = options().run();

    run(options)
}
