//! Support for Qualified Names.

use crate::parser::xml::qname::eqname;
use crate::parser::ParserState;
use crate::trees::nullo::Nullo;
use crate::xdmerror::{Error, ErrorKind};
use core::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::ControlFlow;

#[derive(Clone, Debug)]
pub struct QualifiedName {
    nsuri: Option<String>,
    prefix: Option<String>,
    localname: String,
}

// TODO: we may need methods that return a string slice, rather than a copy of the string
impl QualifiedName {
    pub fn new(
        nsuri: Option<String>,
        prefix: Option<String>,
        localname: impl Into<String>,
    ) -> QualifiedName {
        QualifiedName {
            nsuri,
            prefix,
            localname: localname.into(),
        }
    }
    pub fn as_ref(&self) -> &Self {
        self
    }
    pub fn get_nsuri(&self) -> Option<String> {
        self.nsuri.clone()
    }
    pub fn get_nsuri_ref(&self) -> Option<&str> {
        self.nsuri.as_ref().map(|x| x as _)
    }
    pub fn get_prefix(&self) -> Option<String> {
        self.prefix.clone()
    }
    pub fn get_localname(&self) -> String {
        self.localname.clone()
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        let _ = self.prefix.as_ref().map_or((), |p| {
            result.push_str(p.as_str());
            result.push(':');
        });
        result.push_str(self.localname.as_str());
        f.write_str(result.as_str())
    }
}

pub type QHash<T> = HashMap<QualifiedName, T>;

impl PartialEq for QualifiedName {
    // Only the namespace URI and local name have to match
    fn eq(&self, other: &QualifiedName) -> bool {
        self.nsuri.as_ref().map_or_else(
            || {
                other
                    .nsuri
                    .as_ref()
                    .map_or_else(|| self.localname.eq(other.localname.as_str()), |_| false)
            },
            |ns| {
                other.nsuri.as_ref().map_or_else(
                    || false,
                    |ons| ns.eq(ons.as_str()) && self.localname.eq(other.localname.as_str()),
                )
            },
        )
    }
}
impl Eq for QualifiedName {}

impl Hash for QualifiedName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(ns) = self.nsuri.as_ref() {
            ns.hash(state)
        }
        self.localname.hash(state);
    }
}

/// Parse a string to create a [QualifiedName].
/// QualifiedName ::= (prefix ":")? local-name
impl TryFrom<&str> for QualifiedName {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let state: ParserState<Nullo> = ParserState::new(None, None, None);
        match eqname()((s, state)) {
            Ok((_, qn)) => Ok(qn),
            Err(_) => Err(Error::new(
                ErrorKind::ParseError,
                String::from("unable to parse qualified name"),
            )),
        }
    }
}

/// Parse a string to create a [QualifiedName].
/// Resolve prefix against a set of XML Namespace declarations
/// QualifiedName ::= (prefix ":")? local-name
impl TryFrom<(&str, &Vec<HashMap<String, String>>)> for QualifiedName {
    type Error = Error;
    fn try_from(s: (&str, &Vec<HashMap<String, String>>)) -> Result<Self, Self::Error> {
        let state: ParserState<Nullo> = ParserState::new(None, None, None);
        match eqname()((s.0, state)) {
            Ok((_, qn)) => {
                if qn.get_prefix().is_some() && !qn.get_nsuri_ref().is_some() {
                    match s
                        .1
                        .iter()
                        .try_for_each(|h| match h.get(&qn.get_prefix().unwrap()) {
                            Some(ns) => return ControlFlow::Break(ns.clone()),
                            None => ControlFlow::Continue(()),
                        }) {
                        ControlFlow::Break(ns) => Ok(QualifiedName::new(
                            Some(ns),
                            Some(qn.get_prefix().unwrap()),
                            qn.get_localname(),
                        )),
                        _ => Err(Error::new(
                            ErrorKind::Unknown,
                            format!("unable to match prefix \"{}\"", qn.get_prefix().unwrap()),
                        )),
                    }
                } else {
                    Ok(qn)
                }
            }
            Err(_) => Err(Error::new(
                ErrorKind::ParseError,
                String::from("unable to parse qualified name"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unqualified() {
        assert_eq!(
            QualifiedName::new(None, None, "foo".to_string()).to_string(),
            "foo"
        )
    }
    #[test]
    fn qualified() {
        assert_eq!(
            QualifiedName::new(
                Some("http://example.org/whatsinaname/".to_string()),
                Some("x".to_string()),
                "foo".to_string()
            )
            .to_string(),
            "x:foo"
        )
    }
    #[test]
    fn eqname() {
        let e = QualifiedName::try_from("Q{http://example.org/bar}foo")
            .expect("unable to parse EQName");
        assert_eq!(e.get_localname(), "foo");
        assert_eq!(e.get_nsuri_ref(), Some("http://example.org/bar"));
        assert_eq!(e.get_prefix(), None)
    }
    #[test]
    fn hashmap() {
        let mut h = QHash::<String>::new();
        h.insert(
            QualifiedName::new(None, None, "foo".to_string()),
            String::from("this is unprefixed foo"),
        );
        h.insert(
            QualifiedName::new(
                Some("http://example.org/whatsinaname/".to_string()),
                Some("x".to_string()),
                "foo".to_string(),
            ),
            "this is x:foo".to_string(),
        );
        h.insert(
            QualifiedName::new(
                Some("http://example.org/whatsinaname/".to_string()),
                Some("y".to_string()),
                "bar".to_string(),
            ),
            "this is y:bar".to_string(),
        );

        assert_eq!(h.len(), 3);
        assert_eq!(
            h.get(&QualifiedName {
                nsuri: Some("http://example.org/whatsinaname/".to_string()),
                prefix: Some("x".to_string()),
                localname: "foo".to_string()
            }),
            Some(&"this is x:foo".to_string())
        );
        assert_eq!(
            h.get(&QualifiedName {
                nsuri: None,
                prefix: None,
                localname: "foo".to_string()
            }),
            Some(&"this is unprefixed foo".to_string())
        );
    }
}
