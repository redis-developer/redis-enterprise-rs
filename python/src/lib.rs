//! Python bindings for Redis Enterprise API client

use pyo3::prelude::*;

mod client;
mod error;
mod runtime;

use client::PyEnterpriseClient;
use error::RedisEnterpriseError;

#[pymodule]
fn redis_enterprise(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add(
        "RedisEnterpriseError",
        m.py().get_type::<RedisEnterpriseError>(),
    )?;
    m.add_class::<PyEnterpriseClient>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
