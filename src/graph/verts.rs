use std::ops::{Deref, DerefMut};

use super::Edge;

#[derive(Debug)]
pub struct Vert<VD, EL>
{
    data: VD,
    pub (crate) edges: Vec<Edge<EL>>,
    pub (crate) id: usize,
}

impl<VD, EL> Vert<VD, EL>
{
    // let it only be created through Graph
    pub (crate) fn new(id: usize, data: VD) -> Self
    { Self{ data, id, edges: vec![] } }

    // edges should be added through Graph too
    pub (crate) fn add_edge_with<F>(&mut self, to: usize, producer: F)
        where F: FnOnce() -> EL,
    { self.edges.push(self.edge_with(to, producer)) }


    pub fn id(&self) -> usize
    { self.id }

    pub fn edges_cnt(&self) -> usize
    { self.edges.len() }

// private:
    fn edge_with<F>(&self, to: usize, producer: F) -> Edge<EL>
        where F: FnOnce() -> EL,
    { Edge::new(to, producer()) }
}

impl<VD, EL> Deref for Vert<VD, EL>
{
    type Target = VD;

    fn deref(&self) -> &Self::Target
    { &self.data }
}

impl<VD, EL> DerefMut for Vert<VD, EL>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    { &mut self.data }
}
