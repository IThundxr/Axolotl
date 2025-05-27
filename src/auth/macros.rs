#[macro_export]
macro_rules! define_permissions {
    ( $($ident:ident => $val:expr),* $(,)? ) => {
        impl Permission {
            $(
                pub fn $ident() -> Self {
                    Self::from($val)
                }
            )*
        }
    };
}
