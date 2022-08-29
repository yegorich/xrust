//! # A tree structure for XDM
//!
//! Uses Rc and Weak for a fully navigable tree structure, without using interior mutability.
//!
//! The tree structure has two phases:
//!
//! * Tree construction and mutation - the tree is built and can be mutated, but is not fully navigable. It can only be traversed in a recursive descent.
//! * Tree navigation - the tree is rebuilt using Rc nodes and Weak pointers. The tree is now fully navigable, but cannot be mutated.
//!
//! The first phase uses [ADoc] and [ANode] objects. The second phase uses [BDoc] and [BNode] objects.

use std::convert::TryFrom;
use std::rc::{Rc, Weak};
use std::collections::HashMap;
//use std::marker::PhantomData;
use crate::xdmerror::*;
use crate::qname::*;
use crate::output::OutputDefinition;
use crate::value::Value;
use crate::item::{Document, NodeType, Node};
use crate::rwdocument::{RWDocument, RWNode};
use crate::parsexml::content;

/// Phase A document. These contain [ANode]s.
///
/// A document can have multiple top-level [ANode]s, but to be a well-formed XML document it must have one and only one element-type node.
#[derive(Clone, Default, PartialEq)]
pub struct ADoc {
    pub xmldecl: Option<XMLDecl>,
    pub prologue: Vec<Rc<ANode>>,
    pub content: Vec<Rc<ANode>>,
    pub epilogue: Vec<Rc<ANode>>,
}

impl ADoc {
    fn new() -> Self {
	ADoc{..Default::default()}
    }
    pub fn set_xmldecl(&mut self, x: XMLDecl) {
	self.xmldecl = Some(x)
    }
    pub fn get_xmldecl(&self) -> &Option<XMLDecl> {
	&self.xmldecl
    }
//    fn to_xml(&self) -> String {
//	self.content.iter()
//	    .fold(
//		String::new(),
//		|mut r, c| {
//		    r.push_str(c.to_xml().as_str());
//		    r
//		}
//	    )
//  }
}

pub type RADoc = Rc<ADoc>;

impl RWDocument for RADoc {
    type Docitem = RANode;
    type RWNodeIterator = Box<dyn Iterator<Item = Self::Docitem>>;

    fn push_content(&mut self, n: Self::Docitem) -> Result<(), Error> {
	match Rc::get_mut(self) {
	    Some(d) => Ok(d.content.push(n)),
	    None => return Result::Err(Error::new(ErrorKind::Unknown, String::from("unable to mutate document")))
	}
    }
//    fn push_prologue(&mut self, n: ANode) {
//	self.prologue.push(n)
//    }
//    fn push_epilogue(&mut self, n: ANode) {
//	self.epilogue.push(n)
    //    }
    fn content_iter(&self) -> Self::RWNodeIterator {
	Box::new(ADocChildren::new(self))
    }

    fn new_element(&mut self, qn: QualifiedName) -> Result<Self::Docitem, Error> {
	Ok(Rc::new(
	    ANodeBuilder::new(NodeType::Element)
		.name(qn)
		.build()
	))
    }
    fn new_text(&mut self, v: Value) -> Result<Self::Docitem, Error> {
	Ok(Rc::new(
	    ANodeBuilder::new(NodeType::Text)
		.value(v)
		.build()
	))
    }

    fn to_xml(&self) -> String {
	self.content_iter()
	    .fold(
		String::new(),
		|mut r, c| {
		    r.push_str(c.to_xml().as_str());
		    r
		}
	    )
    }
}

pub struct ADocChildren {
    v: Vec<Rc<ANode>>,
    i: usize,
}
impl ADocChildren {
    fn new(d: &RADoc) -> Self {
	ADocChildren{v: d.content.clone(), i: 0}
    }
}
impl Iterator for ADocChildren {
    type Item = Rc<ANode>;

    fn next(&mut self) -> Option<Self::Item> {
	match self.v.get(self.i) {
	    Some(c) => {
		self.i += 1;
		Some(c.clone())
	    }
	    None => None,
	}
    }
}

pub struct ADocBuilder(ADoc);

impl ADocBuilder {
    pub fn new() -> Self {
	ADocBuilder(ADoc::new())
    }
    pub fn xmldecl(mut self, x: XMLDecl) -> Self {
	self.0.xmldecl = Some(x);
	self
    }
    pub fn prologue(mut self, p: Vec<Rc<ANode>>) -> Self {
	self.0.prologue = p;
	self
    }
    pub fn content(mut self, p: Vec<Rc<ANode>>) -> Self {
	self.0.content = p;
	self
    }
    pub fn epilogue(mut self, p: Vec<Rc<ANode>>) -> Self {
	self.0.epilogue = p;
	self
    }
    pub fn build(self) -> ADoc {
	self.0
    }
}

/// A node in an [ADoc].

#[derive(Clone, Default, PartialEq)]
pub struct ANode {
    node_type: NodeType,
    children: Vec<Rc<ANode>>,
    attributes: HashMap<QualifiedName, Rc<ANode>>,
    name: Option<QualifiedName>,
    value: Option<Value>,
    pi_name: Option<String>,
    dtd: Option<DTDDecl>,
    reference: Option<QualifiedName>,
//    Element(QualifiedName, Vec<ANode>, Vec<ANode>), // Element name, attributes, content
//    Attribute(QualifiedName, Value),
//    Text(Value),
//    PI(String, Value),
//    Comment(Value),	// Comment value is a string
//    DTD(DTDDecl),	// These only occur in the prologue
//    Reference(QualifiedName),	// General entity reference. These need to be resolved before presentation to the application
}

impl ANode {
    fn new(n: NodeType) -> Self {
	ANode{
	    node_type: n,
	    children: vec![],
	    attributes: HashMap::new(),
	    name: None,
	    value: None,
	    pi_name: None,
	    dtd: None,
	    reference: None,
	}
    }

    pub fn name(&self) -> Option<QualifiedName> {
	self.name.clone()
    }
    pub fn value(&self) -> Option<Value> {
	self.value.clone()
    }
    pub fn pi_name(&self) -> Option<String> {
	self.pi_name.clone()
    }
    pub fn reference(&self) -> Option<QualifiedName> {
	self.reference.clone()
    }
}

pub struct ANodeBuilder(ANode);

impl ANodeBuilder {
    pub fn new(n: NodeType) -> Self {
	ANodeBuilder(ANode::new(n))
    }
    pub fn name(mut self, qn: QualifiedName) -> Self {
	self.0.name = Some(qn);
	self
    }
    pub fn value(mut self, v: Value) -> Self {
	self.0.value = Some(v);
	self
    }
    pub fn pi_name(mut self, pi: String) -> Self {
	self.0.pi_name = Some(pi);
	self
    }
    pub fn dtd(mut self, d: DTDDecl) -> Self {
	self.0.dtd = Some(d);
	self
    }
    pub fn reference(mut self, qn: QualifiedName) -> Self {
	self.0.reference = Some(qn);
	self
    }
    pub fn build(self) -> ANode {
	self.0
    }
}

pub type RANode = Rc<ANode>;

impl RWNode for RANode {
    type RWNodeIterator = Box<dyn Iterator<Item = Rc<ANode>>>;

    fn node_type(&self) -> NodeType {
	self.node_type.clone()
    }
    fn name(&self) -> QualifiedName {
	self.name.as_ref().map_or(
	    QualifiedName::new(None, None, String::new()),
	    |n| n.clone()
	)
    }
    fn value(&self) -> Value {
	self.value.as_ref().map_or(
	    Value::from(""),
	    |v| v.clone(),
	)
    }
    fn to_string(&self) -> String {
	String::from("not yet implemented")
    }

    fn to_xml(&self) -> String {
	match self.node_type {
	    NodeType::Element => {
		let mut result = String::from("<");
		result.push_str(self.name().as_ref().to_string().as_str());
		result.push_str(">");
		self.child_iter()
		    .for_each(|c| {
			result.push_str(c.to_xml().as_str())
		    });
		result.push_str("</");
		result.push_str(self.name().as_ref().to_string().as_str());
		result.push_str(">");
		result
	    }
	    NodeType::Text => self.value().to_string(),
	    _ => String::new(),	// TODO
	}
    }

    fn child_iter(&self) -> Self::RWNodeIterator {
	Box::new(ANodeChildren::new(self))
    }

    fn push(&mut self, n: Rc<ANode>) -> Result<(), Error> {
	match Rc::get_mut(self) {
	    Some(p) => {
		p.children.push(n);
		Ok(())
	    }
	    None => Result::Err(Error::new(ErrorKind::Unknown, String::from("unable to mutate node")))
	}
    }
}

pub struct ANodeChildren {
    v: Vec<Rc<ANode>>,
    i: usize,
}
impl ANodeChildren {
    fn new(n: &Rc<ANode>) -> Self {
	match n.node_type() {
	    NodeType::Element => {
		ANodeChildren{v: n.children.clone(), i: 0}
	    }
	    _ => {
		ANodeChildren{v: vec![], i: 0}
	    }
	}
    }
}
impl Iterator for ANodeChildren {
    type Item = Rc<ANode>;

    fn next(&mut self) -> Option<Rc<ANode>> {
	match self.v.get(self.i) {
	    Some(c) => {
		self.i += 1;
		Some(c.clone())
	    }
	    None => None,
	}
    }
}

#[derive(Clone, PartialEq)]
pub struct XMLDecl {
    version: String,
    encoding: Option<String>,
    standalone: Option<String>
}

impl XMLDecl {
    pub fn new(version: String, encoding: Option<String>, standalone: Option<String>) -> Self {
	XMLDecl{version, encoding, standalone}
    }
    pub fn version(&self) -> String {
	self.version.clone()
    }
    pub fn set_encoding(&mut self, e: String) {
	self.encoding = Some(e)
    }
    pub fn encoding(&self) -> String {
	self.encoding.as_ref().map_or(String::new(), |e| e.clone())
    }
    pub fn set_standalone(&mut self, s: String) {
	self.standalone = Some(s)
    }
    pub fn standalone(&self) -> String {
	self.standalone.as_ref().map_or(String::new(), |e| e.clone())
    }
    pub fn to_string(&self) -> String {
	let mut result = String::from("<?xml version=\"");
	result.push_str(self.version.as_str());
	result.push('"');
	self.encoding.as_ref().map(|e| {
	    result.push_str(" encoding=\"");
	    result.push_str(e.as_str());
	    result.push('"');
	});
	self.standalone.as_ref().map(|e| {
	    result.push_str(" standalone=\"");
	    result.push_str(e.as_str());
	    result.push('"');
	});
	result
    }
}

pub struct XMLDeclBuilder(XMLDecl);

impl XMLDeclBuilder {
    pub fn new() -> Self {
	XMLDeclBuilder(XMLDecl{version: String::new(), encoding: None, standalone: None})
    }
    pub fn version(mut self, v: String) -> Self {
	self.0.version = v;
	self
    }
    pub fn encoding(mut self, v: String) -> Self {
	self.0.encoding = Some(v);
	self
    }
    pub fn standalone(mut self, v: String) -> Self {
	self.0.standalone = Some(v);
	self
    }
    pub fn build(self) -> XMLDecl {
	self.0
    }
}

/// DTD declarations.
/// Only general entities are supported, so far.
/// TODO: element, attribute declarations
#[derive(Clone, PartialEq)]
pub enum DTDDecl {
    GeneralEntity(QualifiedName, String),
}

/// The phase 2 Document. Nodes in this type of document are fully navigable, but the tree cannot be mutated.
pub struct BDoc {
//    baseuri: String,
    nodes: Vec<Rc<BNode>>,
//    ph: PhantomData<N>,
}

pub type RBDoc = Rc<BDoc>;

impl BDoc {
    pub fn to_xml(&self) -> String {
	self.nodes.iter()
	    .fold(
		String::new(),
		|mut r, n| {r.push_str(n.to_xml().as_str()); r}
	    )
    }
}

impl Document for RBDoc {
    type Docitem = Rc<BNode>;
    type NodeIterator = Box<dyn Iterator<Item = Self::Docitem>>;

    fn child_iter(&self) -> Self::NodeIterator {
	Box::new(DocChildren::new(self))
    }
}

pub struct DocChildren {
    v: Vec<Rc<BNode>>,
    i: usize,
}
impl DocChildren {
    fn new(d: &Rc<BDoc>) -> Self {
	DocChildren{v: d.nodes.clone(), i: 0}
    }
}

impl Iterator for DocChildren {
    type Item = Rc<BNode>;

    fn next(&mut self) -> Option<Self::Item> {
	match self.v.get(self.i) {
	    Some(c) => {
		self.i += 1;
		Some(c.clone())
	    }
	    None => None,
	}
    }
}

/// Convert an [ADoc], which is mutable but not navigable, to a [BDoc], which is not mutable but is navigable.
///
/// Includes entity expansion.
impl TryFrom<ADoc> for RBDoc {
    type Error = Error;

    fn try_from(a: ADoc) -> Result<Self, Self::Error> {
	let mut ent: HashMap<QualifiedName, Vec<Rc<ANode>>> = HashMap::new();

	// Process general entity declarations and store the result in the HashMap.
	for p in &a.prologue {
	    if p.node_type() == NodeType::Unknown {
		let DTDDecl::GeneralEntity(n, c) = p.dtd.as_ref().unwrap();
		let (rest, e) = content(c.as_str()).map_err(|e| Error::new(ErrorKind::Unknown, e.to_string()))?;
		if rest.len() != 0 {
		    return Result::Err(Error::new(ErrorKind::Unknown, format!("unable to parse general entity \"{}\"", n.to_string())))
		}
		match ent.insert(n.clone(), e) {
		    Some(_) => {
			return Result::Err(Error::new(ErrorKind::Unknown, format!("general entity \"{}\" already defined", n.to_string())))
		    }
		    None => {}
		}
	    }
	}

	Ok(Rc::new_cyclic(|weak_self| {
	    // Descend the A tree, replacing references with their content.
	    // At the same time, convert ANodes to BNodes.
	    let mut new: Vec<Rc<BNode>> = vec![];
	    let mut prologue = a.prologue.into_iter()
		.map(|n| {
		    BNode::from_anode(n, weak_self.clone(), None, &ent)
		})
		.collect();
	    new.append(&mut prologue);
	    let mut content = a.content.into_iter()
		.map(|n| {
		    BNode::from_anode(n, weak_self.clone(), None, &ent)
		})
		.collect();
	    new.append(&mut content);
	    let mut epilogue = a.epilogue.into_iter()
		.map(|n| {
		    BNode::from_anode(n, weak_self.clone(), None, &ent)
		})
		.collect();
	    new.append(&mut epilogue);

	    BDoc{
		//	    baseuri: String::from(""),
		nodes: new,
//		ph: PhantomData,
	    }
	}))
    }
}

/// A node in a phase 2 document, [BDoc].
pub struct BNode {
    doc: Weak<BDoc>,
    node_type: NodeType,
    parent: Option<Weak<BNode>>,
    children: Vec<Rc<BNode>>,
//    attributes: HashMap<QualifiedName, Rc<BNode>>,
    name: Option<QualifiedName>,
    value: Option<Value>,
}

impl BNode {
    fn from_anode(
	n: Rc<ANode>,
	doc: Weak<BDoc>,
	parent: Option<Weak<BNode>>,
	entities: &HashMap<QualifiedName, Vec<Rc<ANode>>>
    ) -> Rc<Self> {
	Rc::new_cyclic(|weak_self| {
	    match n.node_type() {
		// TODO: attributes
		NodeType::Element => {
		    let children: Vec<_> = n.child_iter()
			.map(|child| {
			    BNode::from_anode(child, doc.clone(), Some(weak_self.clone()), entities)
			})
			.collect();
		    BNode{
			doc,
			node_type: NodeType::Element,
			parent, children,
//			attributes: HashMap::new(),
			name: Some(n.name()), value: None
		    }
		}
		NodeType::Attribute => {
		    BNode{
			doc,
			node_type: NodeType::Attribute,
			parent, children: vec![],
//			attributes: HashMap::new(),
			name: Some(n.name()),
			value: Some(n.value())
		    }
		}
		NodeType::Text => {
		    BNode{
			doc,
			node_type: NodeType::Text,
			parent, children: vec![],
//			attributes: HashMap::new(),
			name: None,
			value: Some(n.value())
		    }
		}
		NodeType::ProcessingInstruction => {
		    BNode{
			doc,
			node_type: NodeType::ProcessingInstruction,
			parent, children: vec![],
//			attributes: HashMap::new(),
			name: Some(QualifiedName::new(None, None, n.pi_name().unwrap())),
			value: Some(n.value())
		    }
		}
		NodeType::Comment => {
		    BNode{
			doc,
			node_type: NodeType::Comment,
			parent, children: vec![],
//			attributes: HashMap::new(),
			name: None, value: Some(n.value())
		    }
		}
		// a reference will resolve to a vector of BNodes
		// TODO
		_ => {
		    BNode{
			doc,
			node_type: NodeType::Unknown,
			parent, children: vec![],
//			attributes: HashMap::new(),
			name: None, value: None
		    }
		}
	    }
	})
    }
}

impl Node for Rc<BNode> {
    type NodeIterator = Box<dyn Iterator<Item = Rc<BNode>>>;
    type D = Rc<BDoc>;

    fn owner_document(&self) -> Result<Self::D, Error> {
	Weak::upgrade(&self.doc)
	    .ok_or(Error::new(ErrorKind::Unknown, String::from("unable to find owner document")))
    }

    fn node_type(&self) -> NodeType {
	self.node_type.clone()
    }
    fn name(&self) -> QualifiedName {
	self.name.as_ref().map_or(
	    QualifiedName::new(None, None, String::new()),
	    |n| n.clone()
	)
    }
    fn value(&self) -> Value {
	self.value.as_ref().map_or(
	    Value::from(""),
	    |n| n.clone()
	)
    }
    // String value of the node
    fn to_string(&self) -> String {
	let mut result = String::new();
	match self.node_type {
	    NodeType::Element => {
		self.descend_iter()
		    .filter(|n| n.node_type() == NodeType::Text)
		    .for_each(|n| result.push_str(n.value().to_string().as_str()))
	    }
	    _ => {
		result.push_str(self.value().to_string().as_str())
	    }
	}
	result
    }
    fn to_xml(&self) -> String {
	let mut result = String::new();
	match self.node_type {
	    NodeType::Element => {
		let name = self.name.as_ref().unwrap();
		result.push_str("<");
		result.push_str(name.to_string().as_str());
		result.push_str(">");
		self.children.iter()
		    .for_each(|c| result.push_str(c.to_xml().as_str()));
		result.push_str("</");
		result.push_str(name.to_string().as_str());
		result.push_str(">");
	    }
	    NodeType::Text => {
		result.push_str(self.value.as_ref().unwrap().to_string().as_str())
	    }
	    // TODO: all other types
	    _ => {}
	}
	result
    }
    fn to_xml_with_options(&self, _od: &OutputDefinition) -> String {
	String::from("not yet implemented")
    }
    fn to_json(&self) -> String {
	String::from("not yet implemented")
    }
    fn child_iter(&self) -> Self::NodeIterator {
	Box::new(Children::new(self.clone()))
    }
    fn ancestor_iter(&self) -> Self::NodeIterator {
	Box::new(Ancestors::new(self.clone()))
    }
    fn descend_iter(&self) -> Self::NodeIterator {
	Box::new(Descendants::new(self.clone()))
    }
    fn next_iter(&self) -> Self::NodeIterator {
	Box::new(Siblings::new(self.clone(), 1))
    }
    fn prev_iter(&self) -> Self::NodeIterator {
	Box::new(Siblings::new(self.clone(), -1))
    }
}

pub struct Children {
    v: Vec<Rc<BNode>>,
    i: usize,
}
impl Children {
    fn new(n: Rc<BNode>) -> Self {
	Children{v: n.children.clone(), i: 0}
    }
}
impl Iterator for Children {
    type Item = Rc<BNode>;

    // TODO
    fn next(&mut self) -> Option<Rc<BNode>> {
	match self.v.get(self.i) {
	    Some(c) => {
		self.i += 1;
		Some(c.clone())
	    }
	    None => None,
	}
    }
}

pub struct Ancestors {
    cur: Rc<BNode>,
}

impl Ancestors {
    fn new(n: Rc<BNode>) -> Self {
	Ancestors{cur: n.clone()}
    }
}

impl Iterator for Ancestors {
    type Item = Rc<BNode>;

    fn next(&mut self) -> Option<Rc<BNode>> {
	let p = self.cur.parent.as_ref();
	match p {
	    None => None,
	    Some(q) => {
		match Weak::upgrade(q) {
		    None => None,
		    Some(r) => {
			self.cur = r.clone();
			Some(r)
		    }
		}
	    }
	}
    }
}

// A BDoc is immutable, so the descendants will not change.
// This implementation eagerly constructs a list of nodes
// to traverse.
// An alternative would be to lazily traverse the descendants.
pub struct Descendants{
    v: Vec<Rc<BNode>>,
    cur: usize,
}
impl Descendants {
    fn new(n: Rc<BNode>) -> Self {
	Descendants{
	    v: n.children.iter()
		.fold(
		    vec![],
		    |mut acc, c| {
			let mut d = descendant_add(c);
			acc.append(&mut d);
			acc
		    }
		),
	    cur: 0,
	}
    }
}
fn descendant_add(n: &Rc<BNode>) -> Vec<Rc<BNode>> {
    let mut result = vec![n.clone()];
    n.children.iter()
	.for_each(|c| {
	    let mut l = descendant_add(c);
	    result.append(&mut l);
	});
    result
}
impl Iterator for Descendants {
    type Item = Rc<BNode>;

    fn next(&mut self) -> Option<Rc<BNode>> {
	match self.v.get(self.cur) {
	    Some(n) => {
		self.cur += 1;
		Some(n.clone())
	    }
	    None => None,
	}
    }
}

pub struct Siblings(Rc<BNode>);
impl Siblings {
    fn new(n: Rc<BNode>, _dir: i32) -> Self {
	Siblings(n.clone())
    }
}
impl Iterator for Siblings {
    type Item = Rc<BNode>;

    // TODO
    fn next(&mut self) -> Option<Rc<BNode>> {
	None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_a() {
	eprintln!("make ADoc");
	let ad = Rc::new(
	    ADocBuilder::new()
		.content(vec![
		    Rc::new(
			ANodeBuilder::new(NodeType::Element)
			    .name(QualifiedName::new(None, None, String::from("Test")))
			    .build()
		    )
		])
		.build()
	);
	eprintln!("check XML");
	assert_eq!(ad.to_xml(), "<Test></Test>")
    }
    #[test]
    fn b_from_a() {
	let ad = ADocBuilder::new()
	    .content(vec![
		Rc::new(
		    ANodeBuilder::new(NodeType::Element)
			.name(QualifiedName::new(None, None, String::from("Test")))
			.build()
		)
	    ])
	    .build();
	let bd = RBDoc::try_from(ad).expect("unable to convert ADoc to BDoc");
	assert_eq!(bd.to_xml(), "<Test></Test>")
    }
    #[test]
    fn owner_doc() {
	let ad = ADocBuilder::new()
	    .content(vec![
		Rc::new(
		    ANodeBuilder::new(NodeType::Element)
			.name(QualifiedName::new(None, None, String::from("Test")))
			.build()
		)
	    ])
	    .build();
	let bd = RBDoc::try_from(ad).expect("unable to convert ADoc to BDoc");
	let root_doc = bd.child_iter().nth(0).unwrap().owner_document().expect("unable to get owner document");
	assert!(Rc::ptr_eq(&bd, &root_doc))
    }
    #[test]
    fn b_descend() {
	let mut an1 = Rc::new(
	    ANodeBuilder::new(NodeType::Element)
		.name(QualifiedName::new(None, None, String::from("Test")))
		.build()
	);
	an1.push(Rc::new(
	    ANodeBuilder::new(NodeType::Text)
		.value(Value::from("one-1"))
		.build()
	))
	    .expect("unable to add node");
	let mut an2 = Rc::new(
	    ANodeBuilder::new(NodeType::Element)
		.name(QualifiedName::new(None, None, String::from("Level1")))
		.build()
	);
	let an3 = Rc::new(
	    ANodeBuilder::new(NodeType::Text)
		.value(Value::from("two"))
		.build()
	);
	an2.push(an3)
	    .expect("unable to add node");
	an1.push(an2)
	    .expect("unable to add node");
	let an4 = Rc::new(
	    ANodeBuilder::new(NodeType::Text)
		.value(Value::from("one-2"))
		.build()
	);
	an1.push(an4)
	    .expect("unable to add node");
	let ad = ADocBuilder::new()
	    .content(vec![an1])
	    .build();
	let bd: RBDoc;
	bd = RBDoc::try_from(ad).expect("unable to convert ADoc to BDoc");
	let root = bd.root_element().unwrap();
	let dit = root.descend_iter();
	assert_eq!(dit.count(), 4)
    }
}
