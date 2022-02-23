#[macro_export]
macro_rules! monad_boxed_type {
    ($boxed:ident($type:ty) $(impls $($trait:tt),+)? $(is $($prop:tt),+)?) => {
        paste::paste! {
            #[derive(glib::Boxed, $($($trait),+)?)]
            #[boxed_type(name = "" $boxed "", $($($prop),+)?)]
            pub(crate) struct $boxed($type);
        }

        impl std::ops::Deref for $boxed {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$boxed> for $type {
            fn from(boxed: $boxed) -> Self {
                boxed.0
            }
        }
    };
}
