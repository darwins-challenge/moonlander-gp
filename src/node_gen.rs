#[macro_export]
macro_rules! node_gen {
//----------------------------------------------------------------------
// Internal rules
//
    (@gen_enum $name:$ident { $($cases:tt)* }) => {
        pub enum $name {{ $($cases)* }};
    };

    (@gen_astnode $name:$ident { $($cases:tt)* }) => {
        impl ::moonlander_gp::AstNode for $name {{
            enum $name { $($cases)* };
        }};
    };
    // Eat comma
    (@gen_astnode , $($tail:tt)*) => {
            test!($($tail)*);
        };
    // Eat an identifier with fields
    (@gen_astnode $case:ident ($($fields:tt)*) $($tail:tt)*) => {
            test!($($tail)*);
        };

//----------------------------------------------------------------------
// Public rules
//
    // Initial declaration
    ($name:ident { $($cases:tt)* }) => {
            node_gen!(@gen_enum $name { $($cases)* });
            node_gen!(@gen_astnode $name { $($cases)* });
        };
}
