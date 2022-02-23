#[macro_export]
macro_rules! monad_boxed_type {
    ($boxed:ident($type:ty) $(impls $($trait:tt),+)? $(is $($prop:tt),+)?) => {
        paste::paste! {
            #[derive(glib::Boxed, $($($trait),+)?)]
            #[boxed_type(name = "" $boxed "", $($($prop),+)?)]
            pub(crate) struct $boxed(pub(crate) $type);
        }
    };
}
