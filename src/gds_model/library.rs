use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::HashMap, rc::Rc};

use super::*;
use crate::gds_error;
use crate::{gds_record, gds_writer};

#[derive(Debug)]
struct HashStrucAddr(Rc<RefCell<Struc>>);

impl HashStrucAddr {
    pub fn new(str: &Rc<RefCell<Struc>>) -> Self {
        HashStrucAddr(Rc::clone(str))
    }
}

impl Hash for HashStrucAddr {
    // use Struc obj address as hash
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let borrow = self.0.borrow();
        // borrow.name.hash(state)
        (&*borrow as *const _ as usize).hash(state)
    }
}

impl PartialEq for HashStrucAddr {
    // only use Struc obj address for compare in hash
    fn eq(&self, other: &Self) -> bool {
        let borrow = self.0.borrow();
        let rhs = other.0.borrow();
        let this_addr = &*borrow as *const _ as usize;
        let rhs_addr = &*rhs as *const _ as usize;
        PartialEq::eq(&this_addr, &rhs_addr)
    }
}

impl Eq for HashStrucAddr {}

//  TODO:
// 1. Struc need to knowe about its Lib container, when Struc name changed, Lib need to update
// 2. try use parrellel process to read and write gds file

/// Gds Library
#[derive(Debug)]
pub struct Lib {
    /// Libraray name
    pub name: String,
    /// user units, in meter, points coord in this units，
    ///
    /// for example units is 10e-3,
    /// witch is millimeter，a coord 1.32 means 1.32 millimeter
    ///
    /// default is 1e-6, micron
    pub units: f64,
    /// database units, in meter, gds file double value precision,
    ///
    /// for example
    /// precision is 10e-9, and units is 10e-3, means 6 digit  
    ///
    /// default is 1e-9
    pub precision: f64,
    // each Struc has a uniq name in Lib
    pub(self) strucs: HashSet<HashStrucAddr>,
    pub date: Date,
    pub(self) graph: DiGraph<Rc<RefCell<Struc>>, ()>,
    strucs_nodeidx_map: HashMap<HashStrucAddr, NodeIndex<u32>>,
}

fn get_struc_from_ref(
    refer: &Ref,
    uniq_strucs: &mut HashMap<String, Rc<RefCell<Struc>>>,
    depth: i64,
) {
    let struc = refer.refed_struc.borrow();
    if !uniq_strucs.contains_key(&struc.name) {
        uniq_strucs.insert(struc.name.clone(), refer.refed_struc.clone());
    }
    for r in &struc.refs {
        get_struc_from_ref(r, uniq_strucs, if depth > 0 { depth - 1 } else { depth });
    }
}

impl Lib {
    pub fn new(libname: &str) -> Self {
        Lib {
            name: libname.to_string(),
            units: 1e-6,
            precision: 1e-9,
            strucs: HashSet::<HashStrucAddr>::new(),
            date: Date::now(),
            graph: DiGraph::<Rc<RefCell<Struc>>, ()>::new(),
            strucs_nodeidx_map: HashMap::<HashStrucAddr, NodeIndex<u32>>::new(),
        }
    }

    /// recursely add gds struc to lib
    ///
    /// for example:
    /// struc_a has a ref which refer to struc_b
    ///
    /// lib.add_struc(struc_a) will also add struc_b
    pub fn add_struc(&mut self, struc: Rc<RefCell<Struc>>) -> Result<(), Box<dyn Error>> {
        if self.strucs.contains(&HashStrucAddr::new(&struc)) {
            // Struc has already in Lib, do nothing
        } else {
            self.strucs.insert(HashStrucAddr::new(&struc));
            let nodeidx = self.graph.add_node(struc.clone());
            self.strucs_nodeidx_map
                .insert(HashStrucAddr::new(&struc), nodeidx);
        }
        for r in &struc.borrow().refs {
            let &noedidx = self
                .strucs_nodeidx_map
                .get(&HashStrucAddr::new(&struc))
                .unwrap();
            self.add_struc_helper(r.refed_struc.clone(), &noedidx)?;
        }
        Ok(())
    }

    fn add_struc_helper(
        &mut self,
        struc: Rc<RefCell<Struc>>,
        from_nodeidx: &NodeIndex<u32>,
    ) -> Result<(), Box<dyn Error>> {
        if self.strucs.contains(&HashStrucAddr::new(&struc)) {
            // Struc has already in Lib, do nothing
            let &nodeidx = self
                .strucs_nodeidx_map
                .get(&HashStrucAddr::new(&struc))
                .unwrap();
            self.graph.add_edge(*from_nodeidx, nodeidx, ());
        } else {
            self.strucs.insert(HashStrucAddr::new(&struc));
            let nodeidx = self.graph.add_node(struc.clone());
            self.strucs_nodeidx_map
                .insert(HashStrucAddr::new(&struc), nodeidx);
            self.graph.add_edge(*from_nodeidx, nodeidx, ());
        }
        if is_cyclic_directed(&self.graph) {
            return Err(Box::new(gds_error::gds_err("Ref and Struc make a cycle")));
        }
        for r in &struc.borrow().refs {
            let &nodeidx = self
                .strucs_nodeidx_map
                .get(&HashStrucAddr::new(&struc))
                .unwrap();
            self.add_struc_helper(r.refed_struc.clone(), &nodeidx)?;
        }

        Ok(())
    }

    /// Get Strucs not refered by any Ref
    pub fn top_strucs(&self) -> Vec<Rc<RefCell<Struc>>> {
        let mut top_struc = Vec::<Rc<RefCell<Struc>>>::new(); // self.strucs.clone();
        let mut refed_strucs = HashMap::<String, Rc<RefCell<Struc>>>::new();
        for c in &self.strucs {
            for refer in &c.0.borrow().refs[..] {
                get_struc_from_ref(refer, &mut refed_strucs, -1)
            }
        }
        for ref c in &self.strucs {
            if !refed_strucs.contains_key(&c.0.borrow().name) {
                top_struc.push(c.0.clone());
            }
        }

        top_struc
    }

    /// Get all Strucs
    pub fn all_strucs(&self) -> Vec<Rc<RefCell<Struc>>> {
        self.strucs.iter().map(|c| c.0.clone()).collect::<Vec<_>>()
    }

    /// Dump Lib and recurse dump Lib's Strucs to gds file bytes
    pub fn gds_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.to_gds(0.0)
    }
}

const GDS_VERSIOIN: i16 = 600;

impl GdsObject for Lib {
    fn to_gds(&self, _: f64) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data = Vec::<u8>::new();

        // gds data binary format is in big endian
        // header
        let mut header_data = Vec::<u8>::new();

        header_data.extend(gds_record::HEADER);
        header_data.extend(GDS_VERSIOIN.to_be_bytes());

        data.extend((header_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(header_data);

        // bgnlib and date
        let mut date_data = Vec::<u8>::new();
        date_data.extend(gds_record::BGNLIB);
        date_data.extend(self.date.to_gds(0.0)?);

        data.extend((date_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(date_data);

        // lib name
        let mut lib_name = Vec::<u8>::new();
        lib_name.extend(gds_record::LIBNAME);
        let mut name = gds_writer::ascii_string_to_be_bytes(&self.name);
        if !name.len().is_power_of_two() {
            name.push(0);
        }
        lib_name.extend(name);

        data.extend((lib_name.len() as i16 + 2_i16).to_be_bytes());
        data.extend(lib_name);

        // unit
        let mut unit_data = Vec::<u8>::new();
        unit_data.extend(gds_record::UNITS);
        unit_data.extend(gds_writer::f64_to_gds_bytes(self.precision / self.units));
        unit_data.extend(gds_writer::f64_to_gds_bytes(self.precision));

        data.extend((unit_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(unit_data);

        let scaling = self.units / self.precision;

        // dump strucs
        for ref_c in &self.strucs {
            let struc = ref_c.0.borrow();
            let struc_bytes = struc.to_gds(scaling)?;
            data.extend(struc_bytes);
        }

        // endlib
        let mut endlib_data = Vec::<u8>::new();
        endlib_data.extend(gds_record::ENDLIB);

        data.extend((endlib_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(endlib_data);
        Ok(data)
    }
}

#[cfg(test)]
mod test_lib {
    use super::*;
    #[test]
    fn test_lib() {
        let mut lib = Lib::new("test");
        // make cross ref
        let struc_1 = Rc::new(RefCell::new(Struc::new("test_1")));
        let struc_2 = Rc::new(RefCell::new(Struc::new("test_2")));
        let ref_1 = Ref::new(&struc_1);
        let ref_2 = Ref::new(&struc_2);
        struc_2.borrow_mut().refs.push(ref_1);
        struc_1.borrow_mut().refs.push(ref_2);

        assert!(matches!(lib.add_struc(struc_1), Err(_)));
        assert!(matches!(lib.add_struc(struc_2), Err(_)));
    }
}
