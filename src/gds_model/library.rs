use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableDiGraph;
use petgraph::Direction;

use multi_index_map::MultiIndexMap;

use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use super::*;
use crate::{gds_record, gds_writer};

#[derive(Debug)]
struct HashStrucAddr(Arc<RwLock<Struc>>);

impl HashStrucAddr {
    pub fn new(str: &Arc<RwLock<Struc>>) -> Self {
        HashStrucAddr(Arc::clone(str))
    }
}

impl Hash for HashStrucAddr {
    // use Struc obj address as hash
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let borrow = self.0.read().unwrap();
        (&*borrow as *const _ as usize).hash(state)
    }
}

impl PartialEq for HashStrucAddr {
    // only use Struc obj address for compare in hash
    fn eq(&self, other: &Self) -> bool {
        let borrow = self.0.read().unwrap();
        let rhs = other.0.read().unwrap();
        let this_addr = &*borrow as *const _ as usize;
        let rhs_addr = &*rhs as *const _ as usize;
        PartialEq::eq(&this_addr, &rhs_addr)
    }
}

impl Eq for HashStrucAddr {}

impl Clone for HashStrucAddr {
    fn clone(&self) -> Self {
        HashStrucAddr::new(&self.0)
    }

    fn clone_from(&mut self, source: &Self) {
        self.0 = source.0.clone();
    }
}
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
    pub date: Date,
    pub(self) graph: StableDiGraph<Arc<RwLock<Struc>>, ()>,
    // strucs_nodeidx_map: HashMap<HashStrucAddr, NodeIndex<u32>>,
    uniq_struct: MultiIndexUniqStructMap,
}

#[derive(MultiIndexMap, Debug)]
#[multi_index_derive(Debug)]
pub(crate) struct UniqStruct {
    #[multi_index(hashed_unique)]
    pub(crate) graph_idx: NodeIndex<u32>,
    #[multi_index(hashed_unique)]
    pub(crate) struct_name: String,
    #[multi_index(hashed_unique)]
    struct_address: HashStrucAddr,
}

impl Lib {
    pub fn new(libname: &str) -> Self {
        Lib {
            name: libname.to_string(),
            units: 1e-6,
            precision: 1e-9,
            date: Date::now(),
            graph: StableDiGraph::<Arc<RwLock<Struc>>, ()>::new(),
            uniq_struct: MultiIndexUniqStructMap::default(),
        }
    }

    /// recursely add gds struc to lib
    ///
    /// for example:
    /// struc_a has a ref which refer to struc_b
    ///
    /// lib.add_struc(struc_a) will also add struc_b
    pub fn add_struc(&mut self, struc: &Arc<RwLock<Struc>>) -> Result<(), Box<dyn Error>> {
        // different struct object may have same name, gds formt forbidd same name struct in lib
        if self.diff_struct_has_same_name(&struc) {
            return Err(Box::new(gds_err!(&format!(
                "struc named {} has already existed in lib",
                struc.read().unwrap().name
            ))));
        }
        // check if struc had been added
        match self
            .uniq_struct
            .get_by_struct_address(&HashStrucAddr::new(&struc))
        {
            Some(_) => {
                // if struct had been added before, just recursively add refered strucs
                // do not remove it when add refered strucs failed
                for r in &struc.read().unwrap().refs {
                    self.add_referd_struc(struc.clone(), r.refed_struc.clone())?
                }
                Ok(())
            }
            None => {
                // add struc to graph
                let nodeidx = self.graph.add_node(struc.clone());
                self.uniq_struct.insert(UniqStruct {
                    graph_idx: nodeidx,
                    struct_name: struc.read().unwrap().name.clone(),
                    struct_address: HashStrucAddr::new(&struc),
                });
                // recursly add refered strucs
                for r in &struc.read().unwrap().refs {
                    if let Err(e) = self.add_referd_struc(struc.clone(), r.refed_struc.clone()) {
                        self.uniq_struct.remove_by_graph_idx(&nodeidx);
                        self.graph.remove_node(nodeidx);
                        return Err(e);
                    };
                }
                Ok(())
            }
        }
    }

    fn add_referd_struc(
        &mut self,
        from_struct: Arc<RwLock<Struc>>,
        struc: Arc<RwLock<Struc>>,
    ) -> Result<(), Box<dyn Error>> {
        if is_cyclic_directed(&self.graph) {
            return Err(Box::new(gds_err!(&"circle refer found")));
        }
        if self.diff_struct_has_same_name(&struc) {
            return Err(Box::new(gds_err!(&format!(
                "struc named {} has already existed in lib",
                struc.read().unwrap().name
            ))));
        }
        let from_nodeidx = self
            .uniq_struct
            .get_by_struct_address(&HashStrucAddr::new(&from_struct))
            .unwrap()
            .graph_idx;
        // try add edge from_struc to struc, if struc had been added
        if let Some(uniq_struct) = self
            .uniq_struct
            .get_by_struct_address(&HashStrucAddr::new(&struc))
        {
            let nodeidx = uniq_struct.graph_idx;
            let mut edge_id: Option<EdgeIndex> = None;
            if self.graph.find_edge(from_nodeidx, nodeidx).is_none() {
                edge_id = Some(self.graph.add_edge(from_nodeidx, nodeidx, ()));
            }
            for r in &struc.read().unwrap().refs {
                if let Err(e) = self.add_referd_struc(struc.clone(), r.refed_struc.clone()) {
                    if let Some(edge_idx) = edge_id {
                        self.graph.remove_edge(edge_idx);
                    }
                    return Err(e);
                };
            }
        } else {
            let nodeidx = self.graph.add_node(struc.clone());
            self.uniq_struct.insert(UniqStruct {
                graph_idx: nodeidx,
                struct_name: struc.read().unwrap().name.clone(),
                struct_address: HashStrucAddr::new(&struc),
            });
            self.graph.add_edge(from_nodeidx, nodeidx, ());

            for r in &struc.read().unwrap().refs {
                if let Err(e) = self.add_referd_struc(struc.clone(), r.refed_struc.clone()) {
                    if let Some(uniq_struct) = self.uniq_struct.remove_by_graph_idx(&nodeidx) {
                        // remove node will remove all edges with node
                        self.graph.remove_node(uniq_struct.graph_idx);
                    }

                    return Err(e);
                };
            }
        }

        Ok(())
    }

    /// Remove Struc from Library, won't remove refered strucs
    ///  
    pub fn remove_struc(&mut self, struc: &Arc<RwLock<Struc>>) {
        if let Some(uniq_struc) = self
            .uniq_struct
            .remove_by_struct_address(&HashStrucAddr::new(&struc))
        {
            self.graph.remove_node(uniq_struc.graph_idx);
        }
    }

    fn diff_struct_has_same_name(&self, struc: &Arc<RwLock<Struc>>) -> bool {
        if let Some(same_name_struc) = self
            .uniq_struct
            .get_by_struct_name(&struc.read().unwrap().name)
        {
            if same_name_struc.struct_address != HashStrucAddr::new(&struc) {
                return true;
            }
        }
        return false;
    }

    /// Get Strucs not refered by any Ref
    pub fn top_strucs(&self) -> Vec<Arc<RwLock<Struc>>> {
        let mut top_struc = Vec::<Arc<RwLock<Struc>>>::new();

        for node in self.graph.node_indices() {
            if !self
                .graph
                .neighbors_directed(node, Direction::Incoming)
                .next()
                .is_some()
            {
                top_struc.push(
                    self.uniq_struct
                        .get_by_graph_idx(&node)
                        .unwrap()
                        .struct_address
                        .0
                        .clone(),
                );
            }
        }

        top_struc
    }

    /// Get all Strucs
    pub fn all_strucs(&self) -> Vec<Arc<RwLock<Struc>>> {
        self.uniq_struct
            .iter()
            .map(|c| c.1.struct_address.0.clone())
            .collect::<Vec<_>>()
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
        for (_idx, uniq_struc) in self.uniq_struct.iter() {
            let ref_c = uniq_struc.struct_address.0.clone();
            let struc_bytes = ref_c.read().unwrap().to_gds(scaling)?;
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
    fn test_lib_add_cross_refer_struct_error() {
        let mut lib = Lib::new("test");
        // make cross ref
        let struc_1 = Arc::new(RwLock::new(Struc::new("test_1")));
        let struc_2 = Arc::new(RwLock::new(Struc::new("test_2")));
        let ref_1 = Ref::new(&struc_1);
        let ref_2 = Ref::new(&struc_2);
        struc_2.write().unwrap().refs.push(ref_1);
        struc_1.write().unwrap().refs.push(ref_2);

        // add cross referd struct cause error, lib will be rewinded
        assert!(matches!(lib.add_struc(&struc_1), Err(_)));
        assert!(matches!(lib.add_struc(&struc_2), Err(_)));
        assert!(lib.all_strucs().len() == 0);
    }
    #[test]
    fn test_lib_add_same_name_diff_struct_error() {
        let mut lib = Lib::new("test");
        let struc_1 = Arc::new(RwLock::new(Struc::new("test_1")));
        let struc_2 = Arc::new(RwLock::new(Struc::new("test_1")));
        assert!(matches!(lib.add_struc(&struc_1), Ok(_)));
        assert!(matches!(lib.add_struc(&struc_2), Err(_)));
        assert!(lib.all_strucs().len() == 1);
    }

    #[test]
    fn test_lib_top_struct() {
        let mut lib = Lib::new("test");
        let struc_1 = Arc::new(RwLock::new(Struc::new("test_1")));
        let struc_2 = Arc::new(RwLock::new(Struc::new("test_2")));
        let struc_3 = Arc::new(RwLock::new(Struc::new("test_3")));
        let struc_4 = Arc::new(RwLock::new(Struc::new("test_4")));

        // struc_1 --> struc_3
        let ref_1 = Ref::new(&struc_3);
        struc_1.write().unwrap().refs.push(ref_1);
        // struc_2 --> struc_3
        let ref_2 = Ref::new(&struc_3);
        struc_2.write().unwrap().refs.push(ref_2);
        // struc_3 --> struc_4
        let ref_3 = Ref::new(&struc_4);
        struc_3.write().unwrap().refs.push(ref_3);

        assert!(matches!(lib.add_struc(&struc_1), Ok(_)));
        assert!(matches!(lib.add_struc(&struc_2), Ok(_)));
        assert!(matches!(lib.add_struc(&struc_3), Ok(_)));
        assert!(matches!(lib.add_struc(&struc_4), Ok(_)));
        let top_strucs = lib.top_strucs();
        assert!(top_strucs.len() == 2);

        assert!(top_strucs.iter().any(|v| Arc::ptr_eq(&v, &struc_1)));
        assert!(top_strucs.iter().any(|v| Arc::ptr_eq(&v, &struc_2)));

        assert!(lib.all_strucs().len() == 4);
    }

    #[test]
    fn test_lib_remove_struc() {
        let mut lib = Lib::new("test");
        let struc_1 = Arc::new(RwLock::new(Struc::new("test_1")));
        let struc_2 = Arc::new(RwLock::new(Struc::new("test_2")));
        let ref_1 = Ref::new(&struc_2);
        struc_1.write().unwrap().refs.push(ref_1);
        assert!(matches!(lib.add_struc(&struc_1), Ok(_)));
        assert!(lib.top_strucs().len() == 1);
        assert!(lib.all_strucs().len() == 2);
        // only remove struct1, now struc2 is top struc
        lib.remove_struc(&struc_1);
        let top_s = lib.top_strucs();
        assert!(top_s.len() == 1);
        assert!(Arc::ptr_eq(&top_s[0], &struc_2));
    }
}
