use crate::avm::*;
use crate::grammar::*;
use crate::rewr::*;
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// A chart edge.
pub struct Edge {
    pub start: i32,
    pub end: i32,
    label: String,
    pub logvar: String,
    pub theories: Vec<RewritingSystem>,
    pub level: i32,
    pub used: Cell<bool>,
    children: Vec<Rc<Edge>>,
}

impl Edge {
    /// Creates a new chart edge.
    pub fn new(
        start: i32,
        end: i32,
        label: &str,
        logvar: &str,
        constraints: Vec<Vec<Constraint>>,
    ) -> Self {
        let mut theories = Vec::with_capacity(constraints.len());
        for c in &constraints {
            let mut rs = RewritingSystem::new();
            for c in c {
                let c = c.clone_with_subst(vec![("*".to_owned(), logvar.to_owned())]);
                rs.add_rule(c.rule());
            }
            theories.push(rs);
        }
        Edge::new_with_children(start, end, label, logvar, theories, 0, Vec::new())
    }
    /// Creates a new chart edge spanning daughter edges.
    pub fn new_with_children(
        start: i32,
        end: i32,
        label: &str,
        logvar: &str,
        theories: Vec<RewritingSystem>,
        level: i32,
        children: Vec<Rc<Edge>>,
    ) -> Self {
        Self {
            start: start,
            end: end,
            label: label.to_owned(),
            logvar: logvar.to_owned(),
            theories: theories,
            level: level,
            used: Cell::new(false),
            children: children,
        }
    }
    /// Returns the linearised syntax tree represented by the edge.
    fn tree(&self) -> String {
        let mut tree = self.label.to_owned();
        if self.children.len() > 0 {
            tree += "(";
            tree += &self
                .children
                .iter()
                .map(|e| e.tree())
                .collect::<Vec<_>>()
                .join(",");
            tree += ")";
        }
        tree
    }
}

impl fmt::Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut avms = Vec::with_capacity(self.theories.len());
        for t in &self.theories {
            avms.push(Avm::from_theory(t, &self.logvar));
        }
        write!(
            f,
            "-{}- {} -{}- / {} / {}",
            self.start,
            self.label,
            self.end,
            self.tree(),
            avms.iter()
                .map(|avm| format!("{:?}", avm))
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}

/// A chart for context-free parsing.
pub struct Chart {
    edges: HashMap<i32, Vec<Rc<Edge>>>,
}

impl Chart {
    /// Creates a new empty chart.
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }
    fn find_paths_from(
        &self,
        start: i32,
        pattern: &[Rc<RuleItem>],
        edges: &mut Vec<Rc<Edge>>,
        items: &mut Vec<Rc<RuleItem>>,
        can_skip: bool,
        cb: &mut dyn FnMut(&Vec<Rc<Edge>>, &Vec<Rc<RuleItem>>),
    ) {
        if let Some(item) = pattern.first() {
            if can_skip && item.skippable {
                self.find_paths_from(start, &pattern[1..], edges, items, true, cb);
            }
            if let Some(node_edges) = self.edges.get(&start) {
                for edge in node_edges {
                    if edge.label == item.symbol {
                        edges.push(edge.clone());
                        items.push(item.clone());
                        self.find_paths_from(edge.end, &pattern[1..], edges, items, true, cb);
                        edges.pop();
                        items.pop();
                        if item.repeatable {
                            edges.push(edge.clone());
                            items.push(item.clone());
                            self.find_paths_from(edge.end, pattern, edges, items, false, cb);
                            edges.pop();
                            items.pop();
                        }
                    }
                }
            }
        } else {
            debug_assert!(edges.len() == items.len());
            if edges.len() > 0 {
                cb(edges, items);
            }
        }
    }
    /// Finds all paths in the chart matching the given pattern.
    pub fn find_paths(
        &self,
        pattern: &[Rc<RuleItem>],
        cb: &mut dyn FnMut(&Vec<Rc<Edge>>, &Vec<Rc<RuleItem>>),
    ) {
        let mut edges = Vec::new();
        let mut items = Vec::new();
        for (&i, _) in &self.edges {
            self.find_paths_from(i, pattern, &mut edges, &mut items, true, cb);
        }
    }
    /// Adds a new edge to the chart.
    pub fn add_edge(&mut self, edge: Rc<Edge>) {
        let edges = self.edges.entry(edge.start).or_insert(Vec::new());
        edges.push(edge);
    }
    /// Returns all edges in the chart.
    pub fn all_edges(&self, only_unused: bool) -> Vec<Rc<Edge>> {
        let mut v = Vec::new();
        for (_, edges) in &self.edges {
            for edge in edges {
                if !only_unused || edge.used.get() == false {
                    v.push(edge.clone());
                }
            }
        }
        v.sort_by(|e1, e2| {
            let c = e1.start.cmp(&e2.start);
            if c != Ordering::Equal {
                return c;
            }
            let c = e1.end.cmp(&e2.end);
            if c != Ordering::Equal {
                return c.reverse();
            }
            let c = e1.level.cmp(&e2.level);
            if c != Ordering::Equal {
                return c.reverse();
            }
            e1.label.cmp(&e2.label)
        });
        v
    }
}
