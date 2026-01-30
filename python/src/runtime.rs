//! Tokio runtime management for Python bindings

use pyo3::prelude::*;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("redis-enterprise-py")
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

pub fn block_on<F, T>(py: Python<'_>, future: F) -> T
where
    F: std::future::Future<Output = T> + Send,
    T: Send,
{
    py.allow_threads(|| get_runtime().block_on(future))
}

pub fn future_into_py<'py, F>(py: Python<'py>, future: F) -> PyResult<Bound<'py, PyAny>>
where
    F: std::future::Future<Output = PyResult<Py<PyAny>>> + Send + 'static,
{
    pyo3_async_runtimes::tokio::future_into_py(py, future)
}
