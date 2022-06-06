use std::io::{Read,Error,ErrorKind};
use serde::{Serialize,Deserialize};

pub type PolygonId = usize;

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub enum Level {
    Land,
    Lake,
    IslandInLake,
    PondInIslandInLake,
    Other(u8)
}

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub enum Source {
    CiaWdbii,
    Wvs,
    Other(u8)
}

/// Global Self-consistent Hierarchical High-resolution Shorelines
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Polygon {
    /// Unique polygon id number, starting at 0
    pub id:PolygonId,

    /// Number of points in this polygon
    pub n:usize,

    /// level + version << 8 + greenwich << 16 + source << 24 + river << 25
    pub level:Level,
    pub version:u8,
    pub greenwich_crossed:bool,
    pub source:Source,
    pub river:bool,

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
    pub points:Vec<Point>,

    /// Ids of contained polygons
    pub children:Vec<PolygonId>
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

impl From<u8> for Level {
    fn from(x:u8)->Self {
	match x {
	    1 => Self::Land,
	    2 => Self::Lake,
	    3 => Self::IslandInLake,
	    4 => Self::PondInIslandInLake,
	    _ => Self::Other(x)
	}
    }
}

impl From<u8> for Source {
    fn from(x:u8)->Self {
	match x {
	    0 => Self::CiaWdbii,
	    1 => Self::Wvs,
	    _ => Self::Other(x)
	}
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

	let level = Level::from((flag & 255) as u8);
	let version = ((flag >> 8) & 255) as u8;
	let greenwich_crossed = ((flag >> 16) & 1) != 0;
	let source = Source::from(((flag >> 24) & 255) as u8);
	let river = ((flag >> 24) & 1) != 0;
	
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
	    level,
	    version,
	    greenwich_crossed,
	    source,
	    river,
	    west,
	    east,
	    south,
	    north,
	    area,
	    area_full,
	    container,
	    ancestor,
	    points,
	    children:Vec::new()
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

	// Fill in children
	let npoly = polygons.len();
	for ipoly in 0..npoly {
	    if let Some(iparent) = polygons[ipoly].container {
		polygons[iparent].children.push(ipoly);
	    }
	}

	Ok(Self{
	    polygons
	})
    }
}
