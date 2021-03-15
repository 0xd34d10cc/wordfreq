#![feature(core_intrinsics)]

mod wordcount;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (input, output) = {
        let mut args = std::env::args().skip(1);
        match (args.next(), args.next()) {
            (Some(input), Some(output)) => (input, output),
            _ => {
                eprintln!("Usage: wordfreq <input> <output>");
                return Ok(());
            }
        }
    };

    wordcount::run(&input, &output)?;
    Ok(())
}
