#![no_std]
#![feature(type_alias_impl_trait)]

pub mod app;
pub mod config;
pub mod controllers;
pub mod domain;
pub mod infrastructure;

#[macro_export]
// Create a static cell for a given type and value
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
