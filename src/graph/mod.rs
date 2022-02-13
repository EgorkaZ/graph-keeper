mod verts;
pub use verts::{Vert};

mod edge;
pub use edge::Edge;

mod serialize;
pub use serialize::{to_tgf, Unit};

mod deserialize;
pub use deserialize::read_tgf;

use std::{ops::{Deref, DerefMut}, fmt::{Debug, Formatter, self, Write, Display}, collections::VecDeque};

pub struct Graph<VertData, EdgeLabel>
{
    verts: Vec<Vert<VertData, EdgeLabel>>,
}

impl<VD, EL> Graph<VD, EL>
{
    pub fn new() -> Self
    { Graph{ verts: vec![] } }

// add vert
    pub fn add_vert_with<F>(&mut self, producer: F) -> VertHandleMut<'_, VD, EL>
        where F: FnOnce() -> VD
    {
        let new_id = self.verts.len();
        self.verts.push(Vert::new(new_id, producer()));
        VertHandleMut::new(self, new_id)
    }

    pub fn add_vert(&mut self, data: VD) -> VertHandleMut<'_, VD, EL>
    { self.add_vert_with(move || data) }

    pub fn add_vert_default(&mut self) -> VertHandleMut<'_, VD, EL>
        where VD: Default,
    { self.add_vert_with(VD::default) }

// get vert
    pub fn get_vert(&self, id: usize) -> Option<VertHandle<'_, VD, EL>>
    {
        when! {
            id < self.verts.len() => Some(VertHandle::new(self, id)),
            _ => None
        }
    }

    pub fn get_vert_mut(&mut self, id: usize) -> Option<VertHandleMut<'_, VD, EL>>
    {
        when! {
            id < self.verts.len() => Some(VertHandleMut::new(self, id)),
            _ => None,
        }
    }

    pub fn verts(&self) -> impl Iterator<Item = VertHandle<'_, VD, EL>>
    {
        self.verts.iter()
            .map(|v| VertHandle::new(self, v.id))
    }

    pub fn verts_cnt(&self) -> usize
    { self.verts.len() }

// add edge
    pub fn add_edge_with<F>(&mut self, from: usize, to: usize, producer: F) -> Option<&mut Self>
        where F: FnOnce() -> EL
    {
        let to_id = self.get_vert(to)
            .map(|v_to| v_to.id);

        self.get_vert_mut(from)
            .and_then(|mut v_from| to_id.map(|to_id| v_from.add_edge_with(to_id, producer)))
            .map(|()| self)
    }

    pub fn add_edge(&mut self, from: usize, to: usize, label: EL) -> Option<&mut Self>
    { self.add_edge_with(from, to, move || label) }

// traverse
    pub fn bfs(&self) -> BFSIterator<'_, VD, EL>
    {
        BFSIterator {
            graph: self,
            marked: vec![false; self.verts.len()],
            queue: VecDeque::new(),
            last_root: None
        }
    }

    pub fn bfs_from<'gr>(&'gr self, from: VertHandle<'gr, VD, EL>) -> BFSIterator<'gr, VD, EL>
    {
        assert!(std::ptr::eq(self, from.owner), "'from' is from  different owner");

        let mut marked = vec![false; self.verts.len()];
        marked[from.id] = true;

        let mut queue = VecDeque::new();
        queue.push_back(from);

        BFSIterator {
            graph: self,
            marked,
            queue,
            last_root: None, // 'marked' should be 'false' for all the verts on the left of this idx
        }
    }

// printing
    fn print<DPrinter, LPrinter>(
        &self,
        f: &mut fmt::Formatter<'_>,
        mut print_data: DPrinter,
        mut print_label: LPrinter) -> fmt::Result
        where DPrinter: FnMut(&VD) -> String,
              LPrinter: FnMut(&EL) -> String,
    {
        let mb_print_str = |s: &str, fmt: &mut fmt::Formatter<'_>| {
            if !s.is_empty() {
                fmt.write_fmt(format_args!(" ({s})"))?;
            }
            Ok(())
        };

        self.bfs()
            .try_for_each(|vert| {
                f.write_fmt(format_args!("{}", vert.id))?;
                mb_print_str(&print_data(&vert), f)?;

                f.write_char(':')?;

                vert.edges()
                    .try_for_each(|(label, to)| {
                        mb_print_str(&print_label(label), f)?;
                        f.write_fmt(format_args!(" {},", to.id()))
                    })?;
                f.write_str("\n")
            })
    }
}

impl<VD, EL> Default for Graph<VD, EL>
{
    fn default() -> Self
    { Graph::new() }
}

impl<VD, EL> Extend<VD> for Graph<VD, EL>
{
    fn extend<It>(&mut self, iter: It)
        where It: IntoIterator<Item = VD>
    {
        let fst_new_id = self.verts.len();
        let new_verts = iter.into_iter()
            .enumerate()
            .map(|(id, data)| (id + fst_new_id, data))
            .map(|(id, data)| Vert::new(id, data));

        self.verts.extend(new_verts)
    }
}

impl<VD, EL> Debug for Graph<VD, EL>
    where VD: Debug,
          EL: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        self.print(
            f,
            |data| format!("{:?}", data),
            |label| format!("{:?}", label),
        )
    }
}

impl<VD, EL> Display for Graph<VD, EL>
    where VD: Display,
          EL: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        self.print(
            f,
            VD::to_string,
            EL::to_string,
        )
    }
}

#[derive(Debug)]
pub struct VertHandle<'gr, VD, EL>
{
    owner: &'gr Graph<VD, EL>,
    vert: &'gr Vert<VD, EL>,
}

impl<VD, EL> Clone for VertHandle<'_, VD, EL>
{
    fn clone(&self) -> Self
    { Self { ..*self } }
}

impl<VD, EL> Copy for VertHandle<'_, VD, EL> {}

impl<'gr, VD, EL> VertHandle<'gr, VD, EL>
{
    fn new(owner: &'gr Graph<VD, EL>, id: usize) -> Self
    { VertHandle{ owner, vert: &owner.verts[id] } }

    pub fn edges(&self) -> EdgeIter<'gr, VD, EL>
    { EdgeIter{ graph: self.owner, from: self.vert, edge_idx: 0 } }
}

impl<VD, EL> Deref for VertHandle<'_, VD, EL>
{
    type Target = Vert<VD, EL>;

    fn deref(&self) -> &Self::Target
    { self.vert }
}

#[derive(Debug)]
pub struct VertHandleMut<'gr, VD, EL>
{
    owner: &'gr mut Graph<VD, EL>,
    vert_id: usize,
}

impl<'gr, VD, EL> VertHandleMut<'gr, VD, EL>
{
    fn new(owner: &'gr mut Graph<VD, EL>, vert_id: usize) -> Self
    { VertHandleMut{ owner, vert_id } }

    pub fn owner(&'gr mut self) -> &'gr mut Graph<VD, EL>
    { self.owner }
}

impl<VD, EL> Deref for VertHandleMut<'_, VD, EL>
{
    type Target = Vert<VD, EL>;

    fn deref(&self) -> &Self::Target
    {
        self.owner.verts.get(self.vert_id)
            .expect("VertHandleMut must have been created on valid id")
    }
}

impl<VD, EL> DerefMut for VertHandleMut<'_, VD, EL>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        self.owner.verts.get_mut(self.vert_id)
            .expect("VertHanleMut must have been created on valid id")
    }
}

pub struct EdgeIter<'gr, VD, EL>
{
    graph: &'gr Graph<VD, EL>,
    from: &'gr Vert<VD, EL>,
    edge_idx: usize,
}

impl<'gr, VD, EL> Iterator for EdgeIter<'gr, VD, EL>
{
    type Item = (&'gr EL, VertHandle<'gr, VD, EL>);

    fn next(&mut self) -> Option<Self::Item>
    {
        let res = self.from.edges.get(self.edge_idx)
            .map(|edge| {
                let to_id = edge.to();
                let to = self.graph.get_vert(to_id)
                    .expect("Edge to invalid verticle");
                (edge.label(), to)
            })?;
        self.edge_idx += 1;
        Some(res)
    }
}

pub struct BFSIterator<'gr, VD, EL>
{
    graph: &'gr Graph<VD, EL>,
    marked: Vec<bool>,
    queue: VecDeque<VertHandle<'gr, VD, EL>>,
    last_root: Option<usize>,
}

impl<'gr, VD, EL> Iterator for BFSIterator<'gr, VD, EL>
{
    type Item = VertHandle<'gr, VD, EL>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(from_v) = self.queue.pop_front() {
            let non_marked = from_v.edges()
                .map(|(_, to_v)| to_v)
                .filter(|to_v| {
                    let res = !self.marked[to_v.id];
                    self.marked[to_v.id] = true;
                    res
                });
            self.queue.extend(non_marked);
            Some(from_v)
        } else {
            /* If we don't have any verticies in the queue, then we need to check
                if there is an unvisited connectivity component. For that we'll find
                a non-marked verticle. There are no unvisited verticies before
                'last_root', so let's continue out search of roots after it.
               As we don't check any verticle twice in this search in the whole traversal,
                all out searches will sum to O(V) time, which is no more then the rest
                of the traversal, so won't make it asymptotically worse
            */
            let fst = self.last_root.map(|id| id + 1)
                .unwrap_or(0);

            (fst..)
                .take_while(|id| *id < self.graph.verts.len())
                .find_map(|id| self.graph.get_vert(id)
                    .and_then(|v| when! {
                        !self.marked[v.id] => Some(v),
                        _ => None
                    }))
                .map(|new_root| {
                    let new_root_id = new_root.id;
                    self.last_root = Some(new_root_id);
                    self.marked[new_root_id] = true;
                    new_root
                })
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    type VoidGraph = Graph<(), ()>;
    type UnlabeledGraph<T> = Graph<T, ()>;

    #[test]
    fn add_vert_and_edge() -> Result<(), &'static str>
    {
        let mut graph = VoidGraph::new();
        graph.add_vert(());
        graph.add_vert(());
        graph.add_vert(());

        graph.add_edge(0, 1, ())
            .and_then(|g| g.add_edge(1, 2, ()))
            .and_then(|g| g.add_edge(2, 1, ()))
            .map(|_| ())
            .ok_or("Couldn't add edges")
    }

    #[test]
    fn extend_and_verts() -> Result<(), &'static str>
    {
        let mut graph = UnlabeledGraph::new();
        graph.add_vert("zero");
        graph.extend(["one", "two", "three"]);

        graph.add_edge(0, 2, ())
            .and_then(|g| g.add_edge(1, 3, ()))
            .and_then(|g| g.add_edge(3, 2, ()))
            .and_then(|g| g.add_edge(0, 0, ()))
            .ok_or("Couldn't add edges")?;

        graph.verts()
            .zip(["zero", "one", "two", "three"])
            .for_each(|(v, expected)| {
                let value: &str = &v;
                assert_eq!(value, expected)
            });
        Ok(())
    }

    #[test]
    fn edges_simple() -> Result<(), String>
    {
        let mut graph = Graph::<(), &'static str>::new();
        graph.extend(std::iter::repeat(()).take(5));

        let edges = [(0, "to self"), (1, "to one"), (2, "to two"), (1, "to one v.2"), (4, "to four")];
        edges.into_iter()
            .try_fold(&mut graph, |g, (to, label)| {
                g.add_edge(0, to, label)
                    .ok_or(format!(r#"Couldn't add "{}""#, label))
            })?;

        graph.add_edge(3, 0, "back to zero")
            .and_then(|g| g.add_edge(4, 0, "another to zero"))
            .map(|_| ())
            .ok_or(String::from("Couldn't add edges to 0"))?;

        graph.get_vert(0)
            .map(|v| v.edges()
                .zip(edges)
                .for_each(|((label, to), (expected_id, expected_label))| {
                    assert_eq!(to.id(), expected_id,
                        r#"expected edge: "{}", found: "{}""#, expected_label, label);
                    assert_eq!(*label, expected_label);
                })
            )
            .ok_or(String::from("Couldn't get vert"))
    }

    fn graph_of_size(size: usize) -> VoidGraph
    {
        let mut graph = VoidGraph::new();
        graph.extend(std::iter::repeat(()).take(size));
        graph
    }

    fn test_bfs<VD, EL>(graph: Graph<VD, EL>)
    {
        let mut visited = vec![false; graph.verts_cnt()];

        graph.bfs()
            .for_each(|vert| {
                // check that no verticle is visited twice
                let vert_id = vert.id;
                assert!(!visited[vert_id],
                    "visited {} twice", vert_id);
                visited[vert_id] = true;
            });
        // check no verticle is missed
        visited.into_iter()
            .enumerate()
            .for_each(|(id, is_visited)| {
                assert!(is_visited,
                    "didn't visit {}", id);
            });
    }

    #[test]
    fn bfs_unconnected()
    {
        let graph = graph_of_size(15);
        test_bfs(graph);
    }

    #[test]
    fn bfs_connected()
    {
        let mut graph = graph_of_size(5);
        let edges = [(0, 1), (1, 3), (3, 0), (3, 1), (4, 4), (2, 0), (1, 2)];
        edges.into_iter()
            .fold(&mut graph, |graph, (from, to)| graph.add_edge(from, to, ())
                .expect(&format!("Couldn't add edge from {} to {}", from, to)));

        test_bfs(graph)
    }
}
