use std::error::Error;

mod date;
mod lib;
mod path;
mod points;
mod polygon;
mod sref;
mod struc;
mod text;
mod vector;

pub use self::date::*;
pub use self::lib::*;
pub use self::path::*;
pub use self::points::*;
pub use self::polygon::*;
pub use self::sref::*;
pub use self::struc::*;
pub use self::text::*;
pub use self::vector::*;

trait GdsObject {
    fn to_gds(&self, scaling: f64) -> Result<Vec<u8>, Box<dyn Error>>;
}

#[cfg(test)]
mod test_gds_model {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_lib_top_struc() {
        let mut gds_lib = Lib::new("test");
        let struc1 = Rc::new(RefCell::new(Struc::new("cell1")));
        let struc2 = Rc::new(RefCell::new(Struc::new("cell2")));
        let struc3 = Rc::new(RefCell::new(Struc::new("cell3")));
        let ref3 = Ref::new(struc3.clone());
        let ref2 = Ref::new(struc2.clone());
        struc2.borrow_mut().refs.push(ref3);
        struc1.borrow_mut().refs.push(ref2);
        let _ = gds_lib.add_struc(struc1.clone());
        let _ = gds_lib.add_struc(struc2);
        let _ = gds_lib.add_struc(struc3);

        let top_struc = gds_lib.top_strucs();
        assert_eq!(top_struc.len(), 1);
        assert!(Rc::ptr_eq(&top_struc[0], &struc1));
    }
}
