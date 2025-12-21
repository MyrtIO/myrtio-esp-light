#![no_std]

use core::include_bytes;

pub const FACTORY_PAGE_HTML_GZ: &[u8] = include_bytes!("../dist/index.html.gz");
