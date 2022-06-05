use gshhg_reader::Gshhg;
use std::fs::File;
use std::io::BufReader;

pub fn main() {
    let path = std::env::args().nth(1).expect("Specify path to GSHHG file");
    let fd = File::open(path).expect("Cannot open file");
    let mut buf = BufReader::new(fd);
    let gs = Gshhg::from_reader(&mut buf).expect("Read error");
    println!("{:?}",gs);
}
