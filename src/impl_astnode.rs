/// Macro to implement the AstNode trait
///
/// It's a bit of a bother, so this macro does it for you.
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
/// # impl ::moonlander_gp::RandNode for Tree {
/// #     fn rand(_: &mut ::rand::Rng) -> Tree {
/// #         Tree::Leaf(0)
/// #     }
/// # }
///
/// // Notice sub-trees are indicated by a name, but data fields are
/// // indicated by (data name).
///
/// impl_astnode!(Tree, 666,
///               Leaf((data value)),
///               Node(left, right));
///
/// # fn main() { }
/// ```
#[macro_export]
macro_rules! impl_astnode {
    (@asref $i:ident) => { $i.as_ref() };
    // Field matchers for returning children
    (@retcap (data $i:ident)) => { _ };
    (@retcap $i:ident) => { ref $i };

    // Make vector for all non-data fields
    (@mkvec () ($($acc:expr)*)) => { vec![$($acc),*] };
    (@mkvec ((data $field:ident) $($fields:tt)*) ($($acc:tt)*)) => {
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
    (@repcap (data $i:ident)) => { ref $i };
    (@repcap $i:ident) => { ref $i };
    (@repret $old_child:ident $new_child:ident (data $i:ident)) => { $i.clone() };
    (@repret $old_child:ident $new_child:ident $i:ident) => { $crate::clone_or_replace($i, $old_child, $new_child) };

    // Matching pattern for replacing children, for enum variants with and without parameters
    (@reppat $enum_name:ident $case_name:ident ()) => { $enum_name::$case_name };
    (@reppat $enum_name:ident $case_name:ident ($($fields:tt),+)) => { $enum_name::$case_name($( impl_astnode!(@repcap $fields) ),+) };

    // Constructor call, for enum variants with and without parameters
    (@repcrea $old_child:ident $new_child:ident $enum_name:ident $case_name:ident ()) => { $enum_name::$case_name };
    (@repcrea $old_child:ident $new_child:ident $enum_name:ident $case_name:ident ($($fields:tt),+)) => { $enum_name::$case_name($( impl_astnode!(@repret $old_child $new_child $fields) ),+) };

    ($enum_name:ident, $type_id:expr, $( $case_name:ident ($($fields:tt),*) ),* ) => {
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

            fn replace_child(&self, old_child: &$crate::AstNode, new_child: &mut Option<Box<$crate::AstNode>>) -> Box<$crate::AstNode> {
                Box::new(match *self {
                    $(
                        impl_astnode!(@reppat $enum_name $case_name($($fields),*))
                            =>
                        impl_astnode!(@repcrea old_child new_child $enum_name $case_name($($fields),*))
                    ),*
                })
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
                  Leaf((data d)),
                  Node(left, right));

    impl RandNode for Tree {
        fn rand(_: &mut ::rand::Rng) -> Tree {
            Tree::Leaf(0)
        }
    }

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
