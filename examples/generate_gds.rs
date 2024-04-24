use gdsdk::gds_model::*;
use std::{cell::RefCell, error::Error, io::Write, rc::Rc};

fn main() -> Result<(), Box<dyn Error>> {
    let mut lib = Lib::new("TestLib");

    let cell_a = Rc::new(RefCell::new(Cell::new("cell_a")));

    let mut polygon_1 = Polygon::default();
    // polygon's points no need to be close
    polygon_1.points.push(Points { x: 0.0, y: 0.0 });
    polygon_1.points.push(Points { x: 200.0, y: 0.0 });
    polygon_1.points.push(Points { x: 200.0, y: 100.0 });
    polygon_1.points.push(Points { x: 100.0, y: 100.0 });
    polygon_1.points.push(Points { x: 100.0, y: 200.0 });
    polygon_1.points.push(Points { x: 0.0, y: 200.0 });

    cell_a.borrow_mut().polygons.push(polygon_1);

    let cell_b = Rc::new(RefCell::new(Cell::new("cell_b")));
    let mut polygon_2 = Polygon::default();
    // triangle
    polygon_2.points.push(Points { x: 150., y: 150. });
    polygon_2.points.push(Points { x: 0.0, y: 0.0 });
    polygon_2.points.push(Points { x: 200.0, y: 0.0 });
    cell_b.borrow_mut().polygons.push(polygon_2);
   
    // refer to cell_b
    let mut cell_b_ref = Ref::new(cell_b.clone());
    cell_b_ref.origin = Points::new(300., 300.);
    // cell_b_ref is 3x2 array ref
    cell_b_ref.row = 3;
    cell_b_ref.column = 2;
    cell_b_ref.spaceing_row = Vector::new(400.,50.0);
    cell_b_ref.spaceing_col = Vector::new(50.,400.0);

    cell_a.borrow_mut().refs.push(cell_b_ref);

    lib.add_cell(cell_a);

    let gds_data = lib.gds_bytes()?;

    let mut file = std::fs::File::create("test.gds")?;
    file.write(&gds_data)?;
    Ok(())
}
