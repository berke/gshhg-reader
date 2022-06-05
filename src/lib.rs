use std::io::{Read,Error,ErrorKind};
use serde::{Serialize,Deserialize};

pub type PolygonId = usize;

/// Global Self-consistent Hierarchical High-resolution Shorelines
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Polygon {
    /// Unique polygon id number, starting at 0
    pub id:PolygonId,

    /// Number of points in this polygon
    pub n:usize,

    /// level + version << 8 + greenwich << 16 + source << 24 + river << 25
    pub flag:u32,

    /// Min/max extent in micro-degrees
    pub west:i32,
    pub east:i32,
    pub south:i32,
    pub north:i32,

    /// Area of polygon in 1/10 km^2
    pub area:u32,

    /// Area of original full-resolution polygon in 1/10 km^2
    pub area_full:u32,

    /// Id of container polygon that encloses this polygon
    pub container:Option<PolygonId>,

    /// Id of ancestor polygon in the full resolution set that was the source of this polygon
    pub ancestor:Option<PolygonId>,

    /// Points of the polygon
    pub points:Vec<Point>
}

/// Each lon, lat pair is stored in micro-degrees in 4-byte signed integer format
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Point {
    pub x:i32,
    pub y:i32
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Gshhg {
    pub polygons:Vec<Polygon>
}

fn read_u32<R:Read>(mut r:R)->Result<u32,Error> {
    let mut x = [0;4];
    r.read_exact(&mut x)?;
    Ok(u32::from_be_bytes(x))
}

fn read_i32<R:Read>(mut r:R)->Result<i32,Error> {
    let mut x = [0;4];
    r.read_exact(&mut x)?;
    Ok(i32::from_be_bytes(x))
}

fn read_id_option<R:Read>(r:R)->Result<Option<PolygonId>,Error> {
    let id = read_i32(r)?;
    if id < 0 {
	Ok(None)
    } else {
	Ok(Some(id as usize))
    }
}

impl Point {
    pub fn from_reader<R:Read>(mut r:R)->Result<Self,Error> {
	let x = read_i32(&mut r)?;
	let y = read_i32(&mut r)?;
	Ok(Self{ x,y })
    }
}

impl Polygon {
    pub fn from_reader<R:Read>(mut r:R)->Result<Self,Error> {
	let id = read_id_option(&mut r)?
	    .ok_or_else(|| Error::new(ErrorKind::Other,"Invalid negative ID"))?;
	let n = read_u32(&mut r)? as usize;
	let flag = read_u32(&mut r)?;
	let west = read_i32(&mut r)?;
	let east = read_i32(&mut r)?;
	let south = read_i32(&mut r)?;
	let north = read_i32(&mut r)?;
	let area = read_u32(&mut r)?;
	let area_full = read_u32(&mut r)?;
	let container = read_id_option(&mut r)?;
	let ancestor = read_id_option(&mut r)?;
	let mut points = Vec::with_capacity(n);
	for _ in 0..n {
	    points.push(Point::from_reader(&mut r)?);
	}
	Ok(Self{
	    id,
	    n,
	    flag,
	    west,
	    east,
	    south,
	    north,
	    area,
	    area_full,
	    container,
	    ancestor,
	    points
	})

    }
}

impl Gshhg {
    pub fn from_reader<R:Read>(mut r:R)->Result<Self,Error> {
	let mut polygons = Vec::new();
	loop {
	    match Polygon::from_reader(&mut r) {
		Ok(poly) => polygons.push(poly),
		Err(e) => {
		    match e.kind() {
			ErrorKind::UnexpectedEof => break,
			_ => return Err(e)
		    }
		}
	    }

	}
	Ok(Self{
	    polygons
	})
    }
}
