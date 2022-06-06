use gshhg_reader::{Gshhg,Point};
use std::fs::File;
use std::io::BufReader;

pub fn main() {
    let path = std::env::args().nth(1).expect("Specify path to GSHHG file");
    let fd = File::open(path).expect("Cannot open file");
    let mut buf = BufReader::new(fd);
    let gs = Gshhg::from_reader(&mut buf).expect("Read error");
    for (ipoly,poly) in gs.polygons.iter().enumerate() {
	println!("Polygon {ipoly:8}");
	let (x0,x1,y0,y1) =
	    poly.points.iter()
	    .fold((i32::MAX,i32::MIN,i32::MAX,i32::MIN),
		  |(x0,x1,y0,y1),&Point{ x,y }| (x0.min(x),x1.max(x),y0.min(y),y1.max(y)));
	println!("  Computed bounds [{x0:+10},{x1:+10}] x [{y0:+10},{y1:+10}]");
	println!("  Stored bounds   [{:+10},{:+10}] x [{:+10},{:+10}]",
		 poly.west,
		 poly.east,
		 poly.south,
		 poly.north);
    }
}
