use gdsdk::gds_model::*;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, RwLock};

fn main() -> Result<(), Box<dyn Error>> {
    let mut lib = Lib::new("TestLib");

    let struc_a = Arc::new(RwLock::new(Struc::new("cell_a")));

    let mut polygon_1 = Polygon::default();
    // polygon's points no need to be close
    polygon_1.points.push(Points { x: 0.0, y: 0.0 });
    polygon_1.points.push(Points { x: 200.0, y: 0.0 });
    polygon_1.points.push(Points { x: 200.0, y: 100.0 });
    polygon_1.points.push(Points { x: 100.0, y: 100.0 });
    polygon_1.points.push(Points { x: 100.0, y: 200.0 });
    polygon_1.points.push(Points { x: 0.0, y: 200.0 });

    struc_a.write().unwrap().polygons.push(polygon_1);

    let struc_b = Arc::new(RwLock::new(Struc::new("cell_b")));
    let mut polygon_2 = Polygon::default();
    // triangle
    polygon_2.points.push(Points { x: 150., y: 150. });
    polygon_2.points.push(Points { x: 0.0, y: 0.0 });
    polygon_2.points.push(Points { x: 200.0, y: 0.0 });
    struc_b.write().unwrap().polygons.push(polygon_2);

    // refer to struc_b
    let mut struc_b_ref = Ref::new(&struc_b);
    struc_b_ref.origin = Points::new(300., 300.);
    // struc_b_ref is 3x2 array ref
    struc_b_ref.row = 3;
    struc_b_ref.column = 2;
    struc_b_ref.spaceing_row = Vector::new(400., 50.0);
    struc_b_ref.spaceing_col = Vector::new(50., 400.0);

    struc_a.write().unwrap().refs.push(struc_b_ref);

    let _ = lib.add_struc(&struc_a);

    let gds_data = lib.gds_bytes()?;

    let mut file = std::fs::File::create("test.gds")?;
    file.write_all(&gds_data)?;
    Ok(())
}
