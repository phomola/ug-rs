use crate::chart::*;
use crate::rewr::*;
use std::fmt;
use std::rc::Rc;

/// An item on the right-hand side of a grammar rule.
pub struct RuleItem {
    pub symbol: String,
    pub skippable: bool,
    pub repeatable: bool,
    constraints: Vec<Vec<Constraint>>,
}

impl fmt::Debug for RuleItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.symbol,
            match (self.skippable, self.repeatable) {
                (false, false) => "",
                (true, false) => "?",
                (false, true) => "+",
                (true, true) => "*",
            }
        )
    }
}

/// A morphosyntactic constraint associated with a symbol on the right-hand side of a rule.
#[derive(Clone)]
pub enum Constraint {
    Equal(Term, Term),
}

impl Constraint {
    pub fn clone_with_subst(&self, substs: Vec<(String, String)>) -> Constraint {
        let mut rs = RewritingSystem::new();
        for s in &substs {
            rs.rules.insert(RewriteRule {
                lhs: Term::new(&s.0, &[]),
                rhs: Term::new(&s.1, &[]),
            });
        }
        match self {
            Constraint::Equal(t1, t2) => Constraint::Equal(rs.norm(&t1), rs.norm(&t2)),
        }
    }
    pub fn rule(&self) -> RewriteRule {
        match self {
            Constraint::Equal(t1, t2) => RewriteRule::new(t1.clone(), t2.clone()),
        }
    }
}

impl fmt::Debug for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Equal(t1, t2) => write!(f, "{:?} = {:?}", t1, t2),
        }
    }
}

/// A context-free grammar rule.
pub struct Rule {
    lhs: String,
    rhs: Vec<Rc<RuleItem>>,
}

impl Rule {
    pub fn new(lhs: &str, rhs_ref: &[&str], constraints: Vec<Vec<Vec<Constraint>>>) -> Self {
        let mut rhs = Vec::with_capacity(rhs_ref.len());
        for (i, &s) in rhs_ref.iter().enumerate() {
            let mut skippable = false;
            let mut repeatable = false;
            let s = if let Some(c) = s.chars().last() {
                let mut chars = s.chars();
                match c {
                    '*' => {
                        skippable = true;
                        repeatable = true;
                        chars.next_back();
                    }
                    '+' => {
                        repeatable = true;
                        chars.next_back();
                    }
                    '?' => {
                        skippable = true;
                        chars.next_back();
                    }
                    _ => {}
                }
                chars.as_str().to_owned()
            } else {
                s.to_owned()
            };
            rhs.push(Rc::new(RuleItem {
                symbol: s,
                skippable: skippable,
                repeatable: repeatable,
                constraints: constraints.get(i).unwrap().to_owned(),
            }));
        }
        return Rule {
            lhs: lhs.to_owned(),
            rhs: rhs,
        };
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.lhs,
            self.rhs
                .iter()
                .map(|item| format!("{:?}", item))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

struct ParseContext {
    log_id: i32,
}

impl ParseContext {
    fn unique_id(&mut self) -> i32 {
        self.log_id += 1;
        self.log_id
    }
}

/// A context-free grammar with constraints as rule annotations.
pub struct Grammar {
    rules: Vec<Rule>,
}

impl Grammar {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
    /// Extends the given chart by applying the grammar's rules.
    pub fn parse(&self, chart: &mut Chart) {
        let mut ctx = ParseContext { log_id: 0 };
        self.parse_level(chart, 0, &mut ctx);
    }
    fn parse_level(&self, chart: &mut Chart, level: i32, ctx: &mut ParseContext) {
        let mut new_edges = Vec::new();
        for rule in &self.rules {
            chart.find_paths(&rule.rhs, &mut |edges, items| {
                let mut max_level = 0;
                for edge in edges {
                    if edge.level > max_level {
                        max_level = edge.level;
                    }
                }
                if level == max_level {
                    let mut theories = Vec::new();
                    theories.push(RewritingSystem::new());
                    let new_id = format!("g{}", ctx.unique_id());
                    for (i, edge) in edges.iter().enumerate() {
                        let item = items.get(i).unwrap();
                        let mut new_theories = Vec::new();
                        for rs in &theories {
                            'eloop: for t in &edge.theories {
                                let mut rs = rs.clone();
                                for r in &t.rules {
                                    if !rs.add_rule(r.clone()) {
                                        continue 'eloop;
                                    }
                                }
                                new_theories.push(rs);
                            }
                        }
                        let mut new_theories2 = Vec::new();
                        for rs in &new_theories {
                            'cloop: for c in &item.constraints {
                                let mut rs = rs.clone();
                                for c in c {
                                    let c = c.clone_with_subst(vec![
                                        ("*".to_owned(), new_id.to_owned()),
                                        (".".to_owned(), edge.logvar.to_owned()),
                                    ]);
                                    if !rs.add_rule(c.rule()) {
                                        continue 'cloop;
                                    }
                                }
                                new_theories2.push(rs);
                            }
                        }
                        theories = new_theories2;
                        if theories.is_empty() {
                            break;
                        }
                    }
                    if !theories.is_empty() {
                        for edge in edges {
                            edge.used.set(true);
                        }
                        let edge = Edge::new_with_children(
                            edges.first().unwrap().start,
                            edges.last().unwrap().end,
                            &rule.lhs,
                            &new_id,
                            theories,
                            level + 1,
                            edges.to_owned(),
                        );
                        new_edges.push(Rc::new(edge));
                    }
                }
            });
        }
        if new_edges.len() > 0 {
            for edge in &new_edges {
                chart.add_edge(edge.clone());
            }
            self.parse_level(chart, level + 1, ctx);
        }
    }
}
