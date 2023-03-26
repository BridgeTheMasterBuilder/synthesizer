use anyhow::Result;

// use bpaf::Bpaf;
use instr::run;

// #[derive(Bpaf)]
// #[bpaf(options)]
// struct Options {
//     // #[bpaf(short('i'), long, argument)]
//     // pub input_port: Option<i32>,
//     // #[bpaf(short('l'), long)]
//     // pub list_devices: bool,
//     // #[bpaf(short('o'), long, argument)]
//     // pub output_port: Option<i32>,
// }

fn main() -> Result<()> {
    run()
}
