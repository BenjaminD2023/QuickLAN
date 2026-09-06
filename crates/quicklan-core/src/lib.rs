//! QuickLAN's unprivileged, validated application domain.
pub mod adapter;
pub mod app;
pub mod diagnostics;
pub mod error;
pub mod helper;
pub mod invitation;
pub mod lifecycle;
pub mod model;
pub mod protocol;
pub mod routes;
pub mod runtime;
pub mod storage;
pub mod system_routes;

pub const PRODUCT_NAME: &str = "QuickLAN";
pub const CORE_VERSION: &str = "2.6.4-8428a89d";
pub const PROTOCOL: &str = "easytier-2.6.4-quicklan1";
