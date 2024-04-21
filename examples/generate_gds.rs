use gdsdk::gds_model::*;
use std::{cell::RefCell, error::Error, io::Write, rc::Rc};

fn main() -> Result<(), Box<dyn Error>> {
    let mut lib = Lib::default();
    lib.name = "TestLib".to_string();
    lib.units = 1e-6; // micron
    lib.precision = 1e-9;

    let cell_a = Rc::new(RefCell::new(Cell::default()));

    let mut polygon_1 = Polygon::default();
    // polygon's points no need to be close
    polygon_1.points.push(Points { x: 0.0, y: 0.0 });
    polygon_1.points.push(Points { x: 200.0, y: 0.0 });
    polygon_1.points.push(Points { x: 200.0, y: 100.0 });
    polygon_1.points.push(Points { x: 100.0, y: 100.0 });
    polygon_1.points.push(Points { x: 100.0, y: 200.0 });
    polygon_1.points.push(Points { x: 0.0, y: 200.0 });

    cell_a.borrow_mut().polygons.push(polygon_1);

    lib.cells.push(cell_a);

    let gds_data = lib.gds_bytes()?;

    let mut file = std::fs::File::create("new.gds")?;
    file.write(&gds_data)?;
    Ok(())
}
