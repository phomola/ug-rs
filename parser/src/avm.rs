use crate::rewr::*;
use std::collections::HashMap;
use std::fmt;

/// An AVM value such as an atom or an AVM.
trait AvmValue: fmt::Debug {
    fn as_avm_mut(&mut self) -> Option<&mut Avm> {
        None
    }
}

// A string constant.
struct AvmString {
    value: String,
}

impl AvmValue for AvmString {}

impl AvmString {
    fn new(s: &str) -> Self {
        Self {
            value: s.to_owned(),
        }
    }
}

impl fmt::Debug for AvmString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// An AVM.
pub struct Avm {
    features: HashMap<String, Box<dyn AvmValue>>,
}

impl AvmValue for Avm {
    fn as_avm_mut(&mut self) -> Option<&mut Avm> {
        Some(self)
    }
}

impl Avm {
    fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }
    /// Returns the AVM represented by the given rewriting system.
    pub fn from_theory(rs: &RewritingSystem, logvar: &str) -> Self {
        let logvar = &rs.norm(&Term::new(logvar, &[])).head;
        let mut avm = Self {
            features: HashMap::new(),
        };
        for rule in &rs.rules {
            if rule.lhs.arg != None && rule.lhs.last() == logvar.as_ref() {
                if rule.rhs.head.chars().next().unwrap() == '@' {
                    let mut path = rule.lhs.as_vec();
                    path.reverse();
                    avm.set(&path[1..], Box::new(AvmString::new(&rule.rhs.head[1..])));
                }
            }
        }
        avm
    }
    /// Sets a value for the given path.
    fn set(&mut self, path: &[String], value: Box<dyn AvmValue>) {
        let attr = path.first().unwrap().to_owned();
        if path.len() == 1 {
            self.features.insert(attr, value);
        } else {
            let val = self.features.entry(attr).or_insert(Box::new(Avm::new()));
            if let Some(avm) = val.as_avm_mut() {
                avm.set(&path[1..], value);
            }
        }
    }
}

impl fmt::Debug for Avm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.features)
    }
}
