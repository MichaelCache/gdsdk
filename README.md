# gdsdk
## Breif

gdsii file development kit

read/write gdsii file implemented by rust

## Features
- [x] read gdsii file and parse to gds object(concurrent)
- [x] write gdsii object data to gdsii file(not concurrent right now)
- [x] avoid circular reference of gds object
- [x] create gdsii object like polygons from scratch
- [ ] create/modify gdsii object like polygons by using higher level graphics algorithms

## Usage

### Read

```rust
use gdsdk;

let gds_lib = gdsdk::read_gdsii(&file).unwrap();
```

gds_lib is a `gdsdk::gds_model::Lib` struct, which contains all the data in the gdsii file.

### Write

```rust
use gdsdk::gds_model::*;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, RwLock};

let mut lib = Lib::new("TestLib");
// create a new cell with name "cell_a"
let struc_a = Arc::new(RwLock::new(Struc::new("cell_a")));

let mut polygon_1 = Polygon::default();
// polygon's points no need to be close
polygon_1.points.push(Points { x: 0.0, y: 0.0 });
polygon_1.points.push(Points { x: 200.0, y: 0.0 });
polygon_1.points.push(Points { x: 200.0, y: 100.0 });
polygon_1.points.push(Points { x: 100.0, y: 100.0 });
polygon_1.points.push(Points { x: 100.0, y: 200.0 });
polygon_1.points.push(Points { x: 0.0, y: 200.0 });

// add polygon to cell
struc_a.write().unwrap().polygons.push(polygon_1);

let struc_b = Arc::new(RwLock::new(Struc::new("cell_b")));
let mut polygon_2 = Polygon::default();
polygon_2.points.push(Points { x: 150., y: 150. });
polygon_2.points.push(Points { x: 0.0, y: 0.0 });
polygon_2.points.push(Points { x: 200.0, y: 0.0 });
struc_b.write().unwrap().polygons.push(polygon_2);

// create refer to struc_b
let mut struc_b_ref = Ref::new(&struc_b);
struc_b_ref.origin = Points::new(300., 300.);
// modify struc_b_ref to make a 3x2 array ref
struc_b_ref.row = 3;
struc_b_ref.column = 2;
struc_b_ref.spaceing_row = Vector::new(400., 50.0);
struc_b_ref.spaceing_col = Vector::new(50., 400.0);

// add ref to struc_a
struc_a.write().unwrap().refs.push(struc_b_ref);

// add struc_a will also add it's refs, that is struc_b will be added too
let _ = lib.add_struc(&struc_a);

// dump gds object to gdsii file data
let gds_data = lib.gds_bytes()?;

let mut file = std::fs::File::create("test.gds")?;
file.write_all(&gds_data)?;
```
