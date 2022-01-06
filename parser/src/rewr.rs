use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

/// A unary term.
#[derive(Eq, PartialEq, Clone, Hash)]
pub struct Term {
    pub head: Rc<str>,
    pub arg: Option<Rc<Term>>,
}

impl Term {
    pub fn new(head: &str, comps: &[&str]) -> Self {
        if comps.is_empty() {
            Term {
                head: Rc::from(head),
                arg: None,
            }
        } else {
            Term {
                head: Rc::from(head),
                arg: Some(Rc::new(Term::new(comps[0], &comps[1..]))),
            }
        }
    }
    /// Returns the length of the term.
    pub fn size(&self) -> i32 {
        match &self.arg {
            None => 1,
            Some(arg) => arg.size() + 1,
        }
    }
    /// Returns the innermost argument of the term.
    pub fn last(&self) -> &str {
        match &self.arg {
            None => self.head.as_ref(),
            Some(arg) => arg.last(),
        }
    }
    /// Converts the term into a vector.
    pub fn as_vec(&self) -> Vec<String> {
        let mut v = Vec::new();
        v.push(self.head.as_ref().to_owned());
        if let Some(arg) = &self.arg {
            for el in arg.as_vec() {
                v.push(el);
            }
        }
        v
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Term) -> Option<Ordering> {
        let c = self.size().cmp(&other.size());
        if c != Ordering::Equal {
            return Some(c);
        }
        let c = self.head.cmp(&other.head);
        if c != Ordering::Equal {
            return Some(c);
        }
        if let Some(arg1) = &self.arg {
            if let Some(arg2) = &other.arg {
                return arg1.partial_cmp(arg2);
            }
        }
        Some(Ordering::Equal)
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.arg {
            None => write!(f, "{}", self.head),
            Some(arg) => write!(f, "{}({:?})", self.head, arg),
        }
    }
}

/// A rewrite rule.
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct RewriteRule {
    pub lhs: Term,
    pub rhs: Term,
}

impl RewriteRule {
    pub fn new(t1: Term, t2: Term) -> Self {
        if t1 > t2 {
            RewriteRule { lhs: t1, rhs: t2 }
        } else {
            RewriteRule { lhs: t2, rhs: t1 }
        }
    }
    /// Rewrites the given term.
    pub fn rewrite(&self, t: &Term) -> Option<Term> {
        if t == &self.lhs {
            Some(self.rhs.clone())
        } else {
            match &t.arg {
                None => None,
                Some(arg) => match self.rewrite(arg) {
                    None => None,
                    Some(arg) => Some(Term {
                        head: t.head.clone(),
                        arg: Some(Rc::new(arg)),
                    }),
                },
            }
        }
    }
}

impl fmt::Debug for RewriteRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.lhs, self.rhs)
    }
}

/// A rewriting system.
/// It uses the Knuth-Bendix completion procedure (which is decidable in this special case).
#[derive(Clone)]
pub struct RewritingSystem {
    pub rules: HashSet<RewriteRule>,
}

impl RewritingSystem {
    pub fn new() -> Self {
        Self {
            rules: HashSet::new(),
        }
    }
    /// Returns a norm of the term.
    pub fn norm(&self, t: &Term) -> Term {
        for rule in &self.rules {
            if let Some(t) = &rule.rewrite(t) {
                return self.norm(t);
            }
        }
        t.clone()
    }
    /// Adds a rule to the rewriting system ensuring it remains confluent.
    pub fn add_rule(&mut self, new_rule: RewriteRule) -> bool {
        let t1 = self.norm(&new_rule.lhs);
        let t2 = self.norm(&new_rule.rhs);
        if t1 == t2 {
            return true;
        }
        if t1.arg == None && t2.arg == None {
            if let Some(c1) = t1.head.chars().next() {
                if let Some(c2) = t2.head.chars().next() {
                    if c1 == '@' && c2 == '@' && t1.head != t2.head {
                        return false;
                    }
                }
            }
        }
        let new_rule = RewriteRule::new(t1, t2);
        if self.rules.contains(&new_rule) {
            return true;
        }
        let mut new_rules = Vec::new();
        for rule in &self.rules {
            if let Some(t) = new_rule.rewrite(&rule.lhs) {
                let t1 = self.norm(&t);
                let t2 = self.norm(&rule.rhs);
                if t1 != t2 {
                    new_rules.push(RewriteRule::new(t1.clone(), t2.clone()));
                }
            }
        }
        self.rules.insert(new_rule);
        for rule in new_rules {
            if !self.add_rule(rule) {
                return false;
            }
        }
        true
    }
}

impl fmt::Debug for RewritingSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = "".to_owned();
        for rule in &self.rules {
            s += &format!("{:?}\n", rule);
        }
        write!(f, "{}", s)
    }
}
