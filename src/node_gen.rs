#[macro_export]
macro_rules! impl_astnode {
    // Matching pattern, for enum variants with and without parameters
    (@pat $enum_name:ident, $case_name:ident ()) => { $enum_name::$case_name };
    (@pat $enum_name:ident, $case_name:ident ($($fields:ident),+)) => { $enum_name::$case_name($(ref $fields),+) };

    // Constructor call, for enum variants with and without parameters
    (@create $enum_name:ident, $case_name:ident ()) => { $enum_name::$case_name };
    (@create $enum_name:ident, $case_name:ident ($($fields:expr),+)) => { $enum_name::$case_name($($fields),+) };

    (@children $enum_name:ident, $( $case_name:ident ($($fields:ident),*) ),* ) => {
        fn children(&self) -> Vec<&AstNode> {
            match *self {
                $( impl_astnode!(@pat $enum_name, $case_name($($fields),*)) => vec![$($fields.as_ref()),*] ),*
            }
        }
    };

    ($enum_name:ident, $type_id:expr, $( $case_name:ident ($($fields:ident),*) ),* ) => {
        impl AstNode for $enum_name {
            fn node_type(&self) -> usize { $type_id }

            fn children(&self) -> Vec<&AstNode> {
                match *self {
                    $(
                        impl_astnode!(@pat $enum_name, $case_name($($fields),*))
                            =>
                        vec![$($fields.as_ref()),*]
                    ),*
                }
            }

            fn replace_child(&self, old_child: &AstNode, new_child: &mut Option<Box<AstNode>>) -> Box<AstNode> {
                Box::new(match *self {
                    $(
                        impl_astnode!(@pat $enum_name, $case_name($($fields),*))
                            =>
                        impl_astnode!(@create $enum_name, $case_name($( clone_or_replace($fields, old_child, new_child) ),*))
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
        Leaf,
        Node(Box<Tree>, Box<Tree>)
    }

    impl_astnode!(Tree, 666,
                  Leaf(),
                  Node(left, right));

    impl RandNode for Tree {
        fn rand(_: &mut ::rand::Rng) -> Tree {
            Tree::Leaf
        }
    }

    #[test]
    fn test_children() {
        let node = Tree::Node(Box::new(Tree::Leaf), Box::new(Tree::Leaf));
        assert_eq!(2, node.children().len());
    }

}
