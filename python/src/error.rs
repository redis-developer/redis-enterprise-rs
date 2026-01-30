//! Error handling for Python bindings

use pyo3::exceptions::{PyConnectionError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::{PyErr, create_exception};

create_exception!(
    redis_enterprise,
    RedisEnterpriseError,
    pyo3::exceptions::PyException
);

pub fn enterprise_error_to_py(err: redis_enterprise::RestError) -> PyErr {
    match &err {
        redis_enterprise::RestError::ConnectionError(_) => {
            PyConnectionError::new_err(err.to_string())
        }
        redis_enterprise::RestError::AuthenticationFailed => {
            RedisEnterpriseError::new_err("Authentication failed")
        }
        redis_enterprise::RestError::Unauthorized => {
            RedisEnterpriseError::new_err("Unauthorized access")
        }
        redis_enterprise::RestError::NotFound => PyValueError::new_err("Resource not found"),
        redis_enterprise::RestError::ValidationError(_) => PyValueError::new_err(err.to_string()),
        _ => PyRuntimeError::new_err(err.to_string()),
    }
}

pub trait IntoPyResult<T> {
    fn into_py_result(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for Result<T, redis_enterprise::RestError> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(enterprise_error_to_py)
    }
}
