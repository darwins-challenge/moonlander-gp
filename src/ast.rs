use downcast::Any;
use std::rc::Rc;
use rand::Rng;

//----------------------------------------------------------------------
// AST Traits

/// Main trait to be implemented for AST node types
pub trait AstNode: Any+Mutatable+Copyable+Sync {
    /// Identify the node type, because Any::get_type_id() is unstable.
    fn node_type(&self) -> usize;

    /// Return all children of this node.
    fn children(&self) -> Vec<&AstNode>;

    /// Return a copy of this node with a single child node replaced.
    fn replace_child(&self, old_child: &AstNode, new_child: &mut Option<Box<AstNode>>) -> Box<AstNode>;
}

impl_downcast!(AstNode);
downcast_methods!(AstNode);

/// Mutation trait for nodes
///
/// Implement this if you want to do controlled mutation of nodes.  For example,
/// changing the node type without changing the children, or changing the number
/// value in a controlled way.
///
/// When not explicitly specified, a default implementation is provided for
/// nodes that also implement `RandNode`, which completely replaces the node
/// with a random subtree.
pub trait Mutatable {
    /// Return a mutation of this node
    fn mutate(&self, max_height: i32, rng: &mut Rng) -> Box<AstNode>;
}

/// Like `Clone`, but can be called on a trait object.
///
/// Used during crossover. Automatically implemented for every type that is also
/// `Clone`.
pub trait Copyable {
    /// Like clone(), but unsized. Necessary during crossover.
    fn copy(&self) -> Box<AstNode>;
}

/// Default implementation of Copyable for nodes that are Clone
impl <T: Clone+AstNode> Copyable for T {
    fn copy(&self) -> Box<AstNode> { Box::new(self.clone()) }
}

//----------------------------------------------------------------------
// AST Operations

/// Return the depth of an AST tree.
pub fn depth(node: &AstNode) -> usize {
    1 + node.children().into_iter().map(|c| depth(c)).max().unwrap_or(0)
}

/// A zipper-like structure pointing to a tree node, so a modified copy of the
/// tree can be reconstructed.
#[derive(Clone)]
pub struct NodeInTree<'a> {
    pub node: &'a AstNode,
    pub root_path: Option<Rc<NodeInTree<'a>>>
}

/// Return all nodes in a given AST tree.
pub fn find_nodes_and_parents<'a>(root: &'a AstNode) -> Vec<Rc<NodeInTree<'a>>> {
    let mut result: Vec<Rc<NodeInTree<'a>>> = vec![];
    result.reserve(100);  // Skip some resizes we have to do on medium-sized trees

    let current_root_path = Rc::new(NodeInTree { node: root, root_path: None });
    result.push(current_root_path.clone());
    find_nodes_and_parents_into(root, &current_root_path, &mut result);

    result
}

fn find_nodes_and_parents_into<'a>(parent: &'a AstNode,
                               parent_root_path: &Rc<NodeInTree<'a>>,
                               acc: &mut Vec<Rc<NodeInTree<'a>>>) {
    for node in parent.children() {
        let current_root_path = Rc::new(NodeInTree { node: node.clone(), root_path: Some(parent_root_path.clone()) });
        acc.push(current_root_path.clone());
        find_nodes_and_parents_into(node, &current_root_path, acc);
    }
}

struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

/// Compare a real node and a node trait
fn same_node<T: AstNode>(node1: &T, node2: &AstNode) -> bool {
    use std::mem;
    unsafe {
        let address: *mut() = mem::transmute(node1);
        let obj: TraitObject = mem::transmute(node2);
        address == obj.data
    }
}

/// Helper function for use inside `replace_children()`.
///
/// Call this for every child node in `replace_children()`. The new_child is a
/// `&mut Option<>` so that we can be sure we consume it exactly once.
pub fn clone_or_replace<T: AstNode+Clone>(child: &T, old_child: &AstNode, new_child: &mut Option<Box<AstNode>>) -> Box<T> {
    if same_node(child, old_child) {
        new_child.take().unwrap().downcast::<T>().ok().unwrap()
    } else {
        Box::new(child.clone())
    }
}

/// Return a copy of the entire tree, replacing the indicated node with another.
pub fn replace_to_root<T: AstNode>(nap: &Rc<NodeInTree>, new_child: Box<AstNode>) -> Box<T> {
    let mut new_child_opt = Some(new_child);
    do_replace_to_root(nap, &mut new_child_opt)
}

fn do_replace_to_root<T: AstNode>(nap: &Rc<NodeInTree>, new_child: &mut Option<Box<AstNode>>) -> Box<T> {
    match nap.root_path {
        None => new_child.take().unwrap().downcast().ok().unwrap(),
        Some(ref parent) => {
            let mut new_node = Some(parent.node.replace_child(nap.node, new_child));
            do_replace_to_root(parent, &mut new_node)
        }
    }
}

//----------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[derive(Clone,PartialEq,Eq,Debug)]
    enum TestNode {
        Leaf(u32),
        Node(u32, Box<TestNode>),
        Two(u32, Box<TestNode>, Box<TestNode>)
    }

    impl AstNode for TestNode {
        fn node_type(&self) -> usize { 0 }

        fn children(&self) -> Vec<&AstNode> {
            match *self {
                TestNode::Leaf(_) => vec![],
                TestNode::Node(_, ref x) => vec![x.as_ref()],
                TestNode::Two(_, ref x, ref y) => vec![x.as_ref(), y.as_ref()],
            }
        }

        fn replace_child(&self, old_child: &AstNode, new_child: &mut Option<Box<AstNode>>) -> Box<AstNode> {
            Box::new(match *self {
                TestNode::Leaf(_) => self.clone(),
                TestNode::Node(n, ref x) => TestNode::Node(n,
                                                           clone_or_replace(x, old_child, new_child)),
                TestNode::Two(n, ref x, ref y) => TestNode::Two(n,
                                                                clone_or_replace(x, old_child, new_child),
                                                                clone_or_replace(y, old_child, new_child)),
            })
        }
    }

    impl Mutatable for TestNode {
        fn mutate(&self, _: i32, _: &mut Rng) -> Box<AstNode> {
            Box::new(self.clone())
        }
    }

    fn expect_node(value: u32, ast: &AstNode) {
        let x = ast.downcast_ref::<TestNode>().unwrap();
        if let TestNode::Node(v, _) = *x {
            assert!(v == value);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_find_parents() {
        let tree = Box::new(TestNode::Node(0,
            Box::new(TestNode::Node(1,
                    Box::new(TestNode::Node(2,
                            Box::new(TestNode::Leaf(3))))))));

        let results = find_nodes_and_parents(tree.as_ref());

        expect_node(0, results[0].node);
        assert!(results[0].root_path.is_none());

        expect_node(1, results[1].node);
        expect_node(0, results[1].root_path.as_ref().unwrap().node);

        expect_node(2, results[2].node);
        expect_node(1, results[2].root_path.as_ref().unwrap().node);
        expect_node(0, results[2].root_path.as_ref().unwrap().root_path.as_ref().unwrap().node);
    }

    #[test]
    fn test_replace_child() {
        let tree = TestNode::Two(0,
            Box::new(TestNode::Leaf(1)),
            Box::new(TestNode::Leaf(2)));

        let old_child = tree.children()[1];
        let mut new_child = Some(Box::new(TestNode::Leaf(3)) as Box<AstNode>);
        let new_tree = tree.replace_child(old_child, &mut new_child);

        assert_eq!(&TestNode::Two(0,
            Box::new(TestNode::Leaf(1)),
            Box::new(TestNode::Leaf(3))), new_tree.downcast_ref::<TestNode>().unwrap());
    }

    #[test]
    fn test_depth() {
        let tree = TestNode::Two(0,
            Box::new(TestNode::Leaf(1)),
            Box::new(TestNode::Leaf(2)));

        assert_eq!(2, depth(&tree));
    }
}
