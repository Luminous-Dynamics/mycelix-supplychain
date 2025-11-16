//! HTTP middleware modules
//!
//! Provides cross-cutting concerns for HTTP request handling

pub mod tracing;

pub use self::tracing::trace_request;
