use std::{fs, io::{self, BufReader, BufRead}, env};

use graph_keeper::{read_tgf};

fn main() -> Result<(), String>
{
    let graph = {
        let stdin = io::stdin();

        // if we have filename as first argument,
        // read from it, read from stdin otherwise
        let reader: Box<dyn BufRead> = match env::args().skip(1).next() {
            Some(file_name) => Box::new(BufReader::new(
                fs::File::open(&file_name)
                    .map_err(|err| format!("Couldn't read file {file_name}: {err}"))?
                )),
            None => Box::new(stdin.lock())
        };

        let lines = reader.lines()
            .enumerate()
            .map(|(line_id, mb_line)| mb_line.expect(&format!("Couldn't read line {}", line_id + 1)));

        read_tgf(lines)
            .map_err(|err| err.to_string())?
    };

    print!("{}", graph);

    Ok(())
}
