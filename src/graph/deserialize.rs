use std::{borrow::Borrow, collections::HashMap, fmt::{Debug, Display}, error::Error};

use crate::{Graph};

pub fn read_tgf<It, S>(it: It) -> Result<Graph<String, String>, TGFParseError>
    where It: Iterator<Item = S>,
           S: Borrow<str>
{
    let mut idx_by_name = HashMap::<String, usize>::new();
    let mut graph = Graph::new();

    // skip empty lines
    let mut it = it
        .enumerate()
        .filter(|(_, s)| !s.borrow().trim().is_empty())
        .map(|(line_idx, s)| (line_idx + 1, s));

    // parse verticies
    for (line_num, line) in &mut it {
        let line: &str = line.borrow().trim();

        // Hash sign says that now we should parse edges, also allow empty lines
        if line == "#" {
            break
        }

        // first, parse identifier, here we allow just any non-space characters
        let (name, rest) = name_and_rest(line);

        // No duplicates are allowed, create verticle, then assing it's data
        match idx_by_name.get(name) {
            Some(_) => err_on(line_num, ErrorKind::VerticleDuplicate(name.into())),
            None => {
                let handle = graph.add_vert_with(|| rest.into());
                idx_by_name.insert(name.into(), handle.id);
                Ok(())
            }
        }?;
    }

    // no more mutability needed
    let idx_by_name = idx_by_name;
    // parse edges
    for (line_num, line) in it {
        let line: &str = line.borrow().trim();
        let (from, rest) = name_and_rest(line);
        let (to, label) = name_and_rest(rest);

        if to.is_empty() {
            return err_on(line_num, ErrorKind::EmptyName)
        }

        match (idx_by_name.get(from), idx_by_name.get(to)) {
            (Some(&from_id), Some(&to_id)) => {
                graph.add_edge_with(from_id, to_id, || label.into())
                    .ok_or_else(|| TGFParseError::new(line_num, ErrorKind::InvalidEdge(from.into(), to.into())))
            },
            _ => err_on(line_num, ErrorKind::InvalidEdge(from.into(), to.into()))
        }?;
    }

    Ok(graph)
}

fn name_and_rest(line: &str) -> (&str, &str)
{
    let name_len = bytes_to_ws(line);
    (&line[..name_len], line[name_len..].trim_start())
}

fn bytes_to_ws(line: &str) -> usize
{
    line.chars()
        .take_while(|ch| !ch.is_whitespace())
        .map(|ch| ch.len_utf8())
        .sum()
}

#[derive(Debug)]
pub enum ErrorKind
{
    VerticleDuplicate(String),
    InvalidEdge(String, String),
    EmptyName,
}

impl Display for ErrorKind
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        use ErrorKind::*;
        match self {
            VerticleDuplicate(name) => f.write_fmt(format_args!("Duplicate definition of '{name}'")),
            InvalidEdge(from, to) =>
                f.write_fmt(format_args!("Cannot create edge from '{from}' to '{to}'. At least one of them is be unefined")),
            EmptyName => f.write_str("Empty verticle name is not allowed"),
        }
    }
}

#[derive(Debug)]

pub struct TGFParseError
{
    kind: ErrorKind,
    line: usize,
}

impl TGFParseError
{
    fn new(line_num: usize, kind: ErrorKind) -> Self
    { TGFParseError{ kind, line: line_num } }
}

fn err_on<T>(line_num: usize, kind: ErrorKind) -> Result<T, TGFParseError>
{ Err(TGFParseError::new(line_num, kind)) }

impl Display for TGFParseError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    { f.write_fmt(format_args!("On line {}: {}", self.line, self.kind)) }
}

impl Error for TGFParseError {}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn parse_simple() -> Result<(), String>
    {
        let input = r"
            1
            2
            3
            #
            1 2
            1 3
        ";

        let graph = read_tgf(input.lines())
            .map_err(|err| err.to_string())?;

        assert_eq!(graph.verts_cnt(), 3);
        let start = graph.verts()
            .find(|v| v.edges_cnt() == 2)
            .ok_or(String::from("Couldn't find '1' vert"))?;

        let mut visited = [false; 3];

        graph.bfs_from(start)
            .for_each(|curr| {
                assert!(!visited[curr.id()]);
                visited[curr.id()] = true;

                if curr.id() != start.id() {
                    assert_eq!(curr.edges_cnt(), 0);
                }
            });

        visited.into_iter()
            .enumerate()
            .for_each(|(id, visited)| assert!(visited, "didn't visit {id}"));
        Ok(())
    }

    #[test]
    fn parse_labeled()
    {
        let input = r"
            #1 one edge
            # two edges
            3# unconnected
            #
            #1 # to two
            # #1 to one
            # # to self
        ";
        read_tgf(input.lines())
            .unwrap_or_else(|err| panic!("Parse error: {err}"));
    }

    #[test]
    fn duplicate_error()
    {
        let input =
        r"  1 first
            1 second
            #
            1 1
        ";

        let res = read_tgf(input.lines());
        match res {
            Err(TGFParseError { kind: ErrorKind::VerticleDuplicate(name), line: 2 }) if name == "1" => (),
            Ok(g) => panic!("Unexpected parse: {g}"),
            Err(err) => panic!("Unexpected error: {err}"),
        }
    }

    #[test]
    fn invalid_edge_error()
    {
        let input = 
        r"  one 1
            2 two
            #
            2 one it's valid

            one two it's wrong

            one 2 also valid
        ";

        let res = read_tgf(input.lines());
        match res {
            Err(TGFParseError{
                kind: ErrorKind::InvalidEdge(from, to),
                line: 6,
            }) if from == "one" && to == "two" => (),
            Ok(g) => panic!("Unexpected parse: {g}"),
            Err(err) => panic!("Unexpected error: {err}"),
        }
    }

    #[test]
    fn non_full_edge()
    {
        let input =
        r"  1 one
            2
            3
            #
            1 2 good edge

            1
            3 1 also good
        ";

        let res = read_tgf(input.lines());
        match res {
            Err(TGFParseError{
                kind: ErrorKind::EmptyName,
                line: 7,
            }) => (),
            Ok(g) => panic!("Unexpected parse: {g}"),
            Err(err) => panic!("Unexpected error: {err}"),
        }
    }
}
