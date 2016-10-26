/// Implement AstNode and RandNode trait for a node type.
///
/// Implementing this code for the various variants of an enum
/// is a bit of a bother and quite repetitive, so this macro
/// implements them for you.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate moonlander_gp;
/// # extern crate rand;
///
/// #[derive(Clone,PartialEq,Eq,Debug)]
/// enum Tree {
///     Leaf(i32),
///     Node(Box<Tree>, Box<Tree>)
/// }
///
/// // Notice sub-trees are indicated by a name, but data fields are
/// // indicated by (data name).
///
/// impl_astnode!(Tree, 666,
///               leaf Leaf((data value |rng: &mut ::rand::Rng| (rng.next_u32() % 100) as i32)),
///               int Node(left, right));
///
/// # fn main() { }
/// ```
#[macro_export]
macro_rules! impl_astnode {
    (@asref $i:ident) => { $i.as_ref() };
    // Field matchers for returning children
    (@retcap (data $i:ident $($gen:expr)*)) => { _ };
    (@retcap $i:ident) => { ref $i };

    // Make vector for all non-data fields
    (@mkvec () ($($acc:expr)*)) => { vec![$($acc),*] };
    (@mkvec ((data $field:ident $($gen:expr)*) $($fields:tt)*) ($($acc:tt)*)) => {
        impl_astnode!(@mkvec ($($fields)*) ($($acc)*))
    };
    (@mkvec ($field:ident $($fields:tt)*) ($($acc:tt)*)) => {
        impl_astnode!(@mkvec ($($fields)*) ($($acc)* impl_astnode!(@asref $field)))
    };

    // Matching pattern for returning children, for enum variants with and without parameters
    (@retpat $enum_name:ident $case_name:ident ()) => { $enum_name::$case_name };
    (@retpat $enum_name:ident $case_name:ident ($($fields:tt),+)) => { $enum_name::$case_name($( impl_astnode!(@retcap $fields) ),+) };
    (@retcrea $enum_name:ident $case_name:ident ($($fields:tt),*)) => { impl_astnode!(@mkvec ($($fields)*) ()) };

    // Field matchers for replacing children
    (@repcap (data $i:ident $($gen:expr)*)) => { ref $i };
    (@repcap $i:ident) => { ref $i };
    (@repret $old_child:ident $new_child:ident (data $i:ident $($gen:expr)*)) => { $i.clone() };
    (@repret $old_child:ident $new_child:ident $i:ident) => { $crate::clone_or_replace($i, $old_child, $new_child) };

    // Matching pattern for replacing children, for enum variants with and without parameters
    (@reppat $enum_name:ident $case_name:ident ()) => { $enum_name::$case_name };
    (@reppat $enum_name:ident $case_name:ident ($($fields:tt),+)) => { $enum_name::$case_name($( impl_astnode!(@repcap $fields) ),+) };

    // Constructor call, for enum variants with and without parameters
    (@repcrea $old_child:ident $new_child:ident $enum_name:ident $case_name:ident ()) => { $enum_name::$case_name };
    (@repcrea $old_child:ident $new_child:ident $enum_name:ident $case_name:ident ($($fields:tt),+)) => { $enum_name::$case_name($( impl_astnode!(@repret $old_child $new_child $fields) ),+) };

    // Constructor call, for random variants with and without parameters
    (@randcrea $weights:ident $rng:ident $enum_name:ident $case_name:ident ()) => { $enum_name::$case_name };
    (@randcrea $weights:ident $rng:ident $enum_name:ident $case_name:ident ($($fields:tt),+)) => {
        $enum_name::$case_name($( impl_astnode!(@randchild $weights $rng $fields) ),+)
    };

    // Details for RandNode implementation
    (@callgen $rng:ident) => { "You should pass a random-generating function to a 'data' field" };
    (@callgen $rng:ident $gen:expr) => { $gen($rng) };
    (@randchild $weights:ident $rng:ident (data $field:ident $($gen:expr)*)) => { impl_astnode!(@callgen $rng $($gen)*) };
    (@randchild $weights:ident $rng:ident $field:ident) => { $weights.gen_child($rng) };
    (@weight leaf $weights:expr) => { $weights.leaf() };
    (@weight int $weights:expr) => { $weights.internal() };

    // Entry point
    ($enum_name:ident, $type_id:expr, $( $case_type:ident $case_name:ident ($($fields:tt),*) ),* ) => {
        impl $crate::AstNode for $enum_name {
            fn node_type(&self) -> usize { $type_id }

            fn children(&self) -> Vec<&$crate::AstNode> {
                match *self {
                    $(
                        impl_astnode!(@retpat $enum_name $case_name($($fields),*))
                            =>
                        impl_astnode!(@retcrea $enum_name $case_name($($fields),*))
                    ),*
                }
            }

            fn replace_child(&self, _old_child: &$crate::AstNode, _new_child: &mut Option<Box<$crate::AstNode>>) -> Box<$crate::AstNode> {
                Box::new(match *self {
                    $(
                        impl_astnode!(@reppat $enum_name $case_name($($fields),*))
                            =>
                        impl_astnode!(@repcrea _old_child _new_child $enum_name $case_name($($fields),*))
                    ),*
                })
            }
        }

        impl $crate::RandNode for $enum_name {
            fn rand(weights: $crate::NodeWeights, rng: &mut ::rand::Rng) -> $enum_name {
                pick![rng,
                    $(
                        impl_astnode!(@weight $case_type weights),
                        impl_astnode!(@randcrea weights rng $enum_name $case_name( $( $fields ),* ))
                    ),*
                    ]
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::super::ast::*;

    #[derive(Clone,PartialEq,Eq,Debug)]
    enum Tree {
        Leaf(i32),
        Node(Box<Tree>, Box<Tree>)
    }

    impl_astnode!(Tree, 666,
                  leaf Leaf((data d |rng: &mut ::rand::Rng| (rng.next_u32() % 100) as i32)),
                  int Node(left, right));

    #[test]
    fn test_children() {
        let node = Tree::Node(Box::new(Tree::Leaf(1)), Box::new(Tree::Leaf(2)));
        assert_eq!(2, node.children().len());
    }

    #[test]
    fn copy_data() {
        let node = Tree::Leaf(1);

        // This will not actually replace anything, but test data propagation
        let mut replacement : Option<Box<AstNode>> = Some(Box::new(Tree::Leaf(2)));
        let new_node = node.replace_child(&node, &mut replacement).downcast::<Tree>().ok().unwrap();
        assert_eq!(node, *new_node);
    }

}
