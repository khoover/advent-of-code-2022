use std::io::BufRead;

pub fn get_buffered_input() -> std::io::BufReader<std::fs::File> {
    let mut args = std::env::args();
    args.next().unwrap();
    let input_path = args.next().unwrap();
    let input_file = std::fs::OpenOptions::new()
        .read(true)
        .open(input_path)
        .unwrap();
    std::io::BufReader::new(input_file)
}
