use std::fmt::Display;

use crate::Graph;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// This type can be used for [Graph](crate::Graph) as it implements [Display]
pub struct Unit;

impl Display for Unit
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    { f.write_str("") }
}

struct ToTGF<'gr, VD, EL>(&'gr Graph<VD, EL>);

impl<VD, EL> Display for ToTGF<'_, VD, EL>
    where VD: Display,
          EL: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        self.0.verts()
            .try_for_each(|vert| {
                let data: &VD = &vert;
                f.write_fmt(format_args!("{} {}\n", vert.id() + 1, data))
            })
            .and_then(|_| f.write_str("#\n"))
            .and_then(|_| self.0.verts()
                .try_for_each(|from| from.edges()
                    .try_for_each(|(label, to)| {
                        let from = from.id() + 1;
                        let to = to.id() + 1;
                        f.write_fmt(format_args!("{from} {to} {label}\n"))
                    })
                )
            )
    }
}

pub fn to_tgf<VD, EL>(graph: &Graph<VD, EL>) -> String
    where VD: Display,
          EL: Display,
{ format!("{}", ToTGF(graph)) }
