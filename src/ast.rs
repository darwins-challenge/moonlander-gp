use downcast_rs::Downcast;
use std::rc::Rc;
use rand::Rng;

//----------------------------------------------------------------------
// AST Traits

pub trait AstNode: Downcast+Mutatable {
    /// Identify the node type, because we can't use get_type_id().
    fn node_type(&self) -> usize;

    /// Return all children of this node
    fn children(&self) -> Vec<Rc<AstNode>>;

    fn replace_child(&self, old_child: &AstNode, new_child: &AstNode) -> Box<AstNode>;
}

impl_downcast!(AstNode);

pub trait Mutatable {
    /// Return a mutation of this node
    fn mutate(&self, rng: &mut Rng) -> Box<AstNode>;
}

/// A trait like rand::Rand, but one that doesn't require that the Rng instance is Sized,
/// so that we can combine it with the Mutatable trait.
///
/// We also will encode relative weights for tree depth into here. If the node has a choice
/// between generating an internal node or a leaf node, it should use these weights.
pub trait RandNode: Sized {
    fn rand(rng: &mut Rng) -> Self;
}

impl <T: RandNode+AstNode> Mutatable for T {
    fn mutate(&self, rng: &mut Rng) -> Box<AstNode> {
        Box::new(T::rand(rng))
    }
}

//----------------------------------------------------------------------
// AST Operations

pub fn depth(node: &Rc<AstNode>) -> usize {
    1 + node.children().iter().map(|c| depth(c)).max().unwrap()
}

#[derive(Clone)]
pub struct NodeAndParent {
    pub node: Rc<AstNode>,
    pub root_path: Option<Rc<NodeAndParent>>
}

pub fn find_nodes_and_parents(root: Rc<AstNode>) -> Vec<Rc<NodeAndParent>> {
    let mut result: Vec<Rc<NodeAndParent>> = vec![];
    result.reserve(100);  // Skip some resizes we have to do on medium-sized trees

    let current_root_path = Rc::new(NodeAndParent { node: root.clone(), root_path: None });
    result.push(current_root_path.clone());
    find_nodes_and_parents_into(root, &current_root_path, &mut result);

    result
}

fn find_nodes_and_parents_into(parent: Rc<AstNode>,
                               parent_root_path: &Rc<NodeAndParent>,
                               acc: &mut Vec<Rc<NodeAndParent>>) {
    for node in parent.children() {
        let current_root_path = Rc::new(NodeAndParent { node: node.clone(), root_path: Some(parent_root_path.clone()) });
        acc.push(current_root_path.clone());
        find_nodes_and_parents_into(node, &current_root_path, acc);
    }
}

fn ref_eq<T: ?Sized>(a: &T, b: &T) -> bool {
    a as *const T == b as *const T
}

pub fn clone_or_replace<T: AstNode+Sized+Clone>(rc: &Rc<T>, old_child: &AstNode, new_child: &AstNode) -> Rc<T> {
    if ref_eq(rc.as_ref() as &AstNode, old_child) {
        Rc::new(new_child.downcast_ref::<T>().unwrap().clone())
    } else {
        rc.clone()
    }
}

pub fn replace_to_root<T: AstNode+Clone>(nap: &Rc<NodeAndParent>, new_child: &AstNode) -> Rc<T> {
    match nap.root_path {
        None => Rc::new(new_child.downcast_ref::<T>().unwrap().clone()),
        Some(ref parent_rc) => {
            let new_node = parent_rc.node.replace_child(nap.node.as_ref(), new_child);
            replace_to_root(parent_rc, new_node.as_ref())
        }
    }
}

//----------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use super::*;
    use rand::Rng;

    #[derive(Clone,PartialEq,Eq,Debug)]
    enum TestNode {
        Leaf(u32),
        Node(u32, Rc<TestNode>),
        Two(u32, Rc<TestNode>, Rc<TestNode>)
    }

    impl AstNode for TestNode {
        fn node_type(&self) -> usize { 0 }

        fn children(&self) -> Vec<Rc<AstNode>> {
            match *self {
                TestNode::Leaf(_) => vec![],
                TestNode::Node(_, ref x) => vec![x.clone()],
                TestNode::Two(_, ref x, ref y) => vec![x.clone(), y.clone()],
            }
        }

        fn replace_child(&self, old_child: &AstNode, new_child: &AstNode) -> Box<AstNode> {
            Box::new(match *self {
                TestNode::Leaf(_) => self.clone(),
                TestNode::Node(n, ref x) => TestNode::Node(n, clone_or_replace(x, old_child, new_child)),
                TestNode::Two(n, ref x, ref y) => TestNode::Two(n, clone_or_replace(x, old_child, new_child), clone_or_replace(y, old_child, new_child)),
            })
        }
    }

    impl Mutatable for TestNode {
        fn mutate(&self, _: &mut Rng) -> Box<AstNode> {
            Box::new(self.clone())
        }
    }

    fn expect_node(value: u32, ast: &Rc<AstNode>) {
        let x = ast.downcast_ref::<TestNode>().unwrap();
        if let TestNode::Node(v, _) = *x {
            assert!(v == value);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_find_parents() {
        let tree = Rc::new(TestNode::Node(0,
            Rc::new(TestNode::Node(1,
                    Rc::new(TestNode::Node(2,
                            Rc::new(TestNode::Leaf(3))))))));

        let results = find_nodes_and_parents(tree);

        expect_node(0, &results[0].node);
        assert!(results[0].root_path.is_none());

        expect_node(1, &results[1].node);
        expect_node(0, &results[1].root_path.as_ref().unwrap().node);

        expect_node(2, &results[2].node);
        expect_node(1, &results[2].root_path.as_ref().unwrap().node);
        expect_node(0, &results[2].root_path.as_ref().unwrap().root_path.as_ref().unwrap().node);
    }

    #[test]
    fn test_replace_child() {
        let tree = TestNode::Two(0,
            Rc::new(TestNode::Leaf(1)),
            Rc::new(TestNode::Leaf(2)));

        let new_tree = if let TestNode::Two(_, _, x) = tree.clone() {
            tree.replace_child(x.as_ref(), &TestNode::Leaf(3))
        } else {
            Box::new(TestNode::Leaf(666)) as Box<AstNode>
        };

        assert_eq!(&TestNode::Two(0,
            Rc::new(TestNode::Leaf(1)),
            Rc::new(TestNode::Leaf(3))), new_tree.downcast_ref::<TestNode>().unwrap());
    }
}
