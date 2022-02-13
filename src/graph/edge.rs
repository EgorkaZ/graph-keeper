
#[derive(Debug)]
pub struct Edge<EL>
{
    pub (crate) to: usize,
    label: EL,
}

impl<EL> Edge<EL>
{
    // can only be created through Graph
    pub (crate) fn new(to: usize, label: EL) -> Self
    { Edge{ to, label } }

    pub fn label(&self) -> &EL
    { &self.label }

    pub fn to(&self) -> usize
    { self.to }
}
