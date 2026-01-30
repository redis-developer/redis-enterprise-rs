//! Python bindings for Redis Enterprise API client

use crate::error::IntoPyResult;
use crate::runtime::{block_on, future_into_py};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use redis_enterprise::{BdbHandler, ClusterHandler, EnterpriseClient, NodeHandler, UserHandler};
use std::sync::Arc;
use std::time::Duration;

/// Redis Enterprise API client
#[pyclass(name = "EnterpriseClient")]
pub struct PyEnterpriseClient {
    client: Arc<EnterpriseClient>,
}

#[pymethods]
impl PyEnterpriseClient {
    /// Create a new Redis Enterprise client
    #[new]
    #[pyo3(signature = (base_url, username, password, insecure=false, timeout_secs=None))]
    fn new(
        base_url: String,
        username: String,
        password: String,
        insecure: bool,
        timeout_secs: Option<u64>,
    ) -> PyResult<Self> {
        let mut builder = EnterpriseClient::builder()
            .base_url(base_url)
            .username(username)
            .password(password)
            .insecure(insecure);

        if let Some(secs) = timeout_secs {
            builder = builder.timeout(Duration::from_secs(secs));
        }

        let client = builder.build().into_py_result()?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Create client from environment variables
    #[staticmethod]
    fn from_env() -> PyResult<Self> {
        let client = EnterpriseClient::from_env().into_py_result()?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    // Cluster API

    /// Get cluster information (async)
    fn cluster_info<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = ClusterHandler::new((*client).clone());
            let info = handler.info().await.into_py_result()?;
            let json = serde_json::to_value(&info)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// Get cluster information (sync)
    fn cluster_info_sync(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = ClusterHandler::new((*client).clone());
            handler.info().await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    /// Get cluster statistics (async)
    fn cluster_stats<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = ClusterHandler::new((*client).clone());
            let stats = handler.stats().await.into_py_result()?;
            Python::with_gil(|py| Ok(json_to_py(py, stats)))
        })
    }

    /// Get cluster statistics (sync)
    fn cluster_stats_sync(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = ClusterHandler::new((*client).clone());
            handler.stats().await.into_py_result()
        })?;
        Ok(json_to_py(py, result))
    }

    /// Get license information (async)
    fn license<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = ClusterHandler::new((*client).clone());
            let license = handler.license().await.into_py_result()?;
            let json = serde_json::to_value(&license)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// Get license information (sync)
    fn license_sync(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = ClusterHandler::new((*client).clone());
            handler.license().await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    // Databases API

    /// List all databases (async)
    fn databases<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = BdbHandler::new((*client).clone());
            let dbs = handler.list().await.into_py_result()?;
            let json = serde_json::to_value(&dbs)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// List all databases (sync)
    fn databases_sync(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = BdbHandler::new((*client).clone());
            handler.list().await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    /// Get a specific database by ID (async)
    fn database<'py>(&self, py: Python<'py>, uid: u32) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = BdbHandler::new((*client).clone());
            let db = handler.get(uid).await.into_py_result()?;
            let json = serde_json::to_value(&db)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// Get a specific database by ID (sync)
    fn database_sync(&self, py: Python<'_>, uid: u32) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = BdbHandler::new((*client).clone());
            handler.get(uid).await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    // Nodes API

    /// List all nodes (async)
    fn nodes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = NodeHandler::new((*client).clone());
            let nodes = handler.list().await.into_py_result()?;
            let json = serde_json::to_value(&nodes)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// List all nodes (sync)
    fn nodes_sync(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = NodeHandler::new((*client).clone());
            handler.list().await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    /// Get a specific node by ID (async)
    fn node<'py>(&self, py: Python<'py>, uid: u32) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = NodeHandler::new((*client).clone());
            let node = handler.get(uid).await.into_py_result()?;
            let json = serde_json::to_value(&node)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// Get a specific node by ID (sync)
    fn node_sync(&self, py: Python<'_>, uid: u32) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = NodeHandler::new((*client).clone());
            handler.get(uid).await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    // Users API

    /// List all users (async)
    fn users<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let handler = UserHandler::new((*client).clone());
            let users = handler.list().await.into_py_result()?;
            let json = serde_json::to_value(&users)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| Ok(json_to_py(py, json)))
        })
    }

    /// List all users (sync)
    fn users_sync(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            let handler = UserHandler::new((*client).clone());
            handler.list().await.into_py_result()
        })?;
        let json = serde_json::to_value(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(json_to_py(py, json))
    }

    // Raw API access

    /// Execute a raw GET request (async)
    fn get<'py>(&self, py: Python<'py>, path: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let result = client.get_raw(&path).await.into_py_result()?;
            Python::with_gil(|py| Ok(json_to_py(py, result)))
        })
    }

    /// Execute a raw GET request (sync)
    fn get_sync(&self, py: Python<'_>, path: String) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(
            py,
            async move { client.get_raw(&path).await.into_py_result() },
        )?;
        Ok(json_to_py(py, result))
    }

    /// Execute a raw POST request (async)
    fn post<'py>(
        &self,
        py: Python<'py>,
        path: String,
        body: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let body_json = py_to_json(py, body)?;
        let client = self.client.clone();
        future_into_py(py, async move {
            let result = client.post_raw(&path, body_json).await.into_py_result()?;
            Python::with_gil(|py| Ok(json_to_py(py, result)))
        })
    }

    /// Execute a raw POST request (sync)
    fn post_sync(&self, py: Python<'_>, path: String, body: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let body_json = py_to_json(py, body)?;
        let client = self.client.clone();
        let result = block_on(py, async move {
            client.post_raw(&path, body_json).await.into_py_result()
        })?;
        Ok(json_to_py(py, result))
    }

    /// Execute a raw DELETE request (async)
    fn delete<'py>(&self, py: Python<'py>, path: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let result = client.delete_raw(&path).await.into_py_result()?;
            Python::with_gil(|py| Ok(json_to_py(py, result)))
        })
    }

    /// Execute a raw DELETE request (sync)
    fn delete_sync(&self, py: Python<'_>, path: String) -> PyResult<Py<PyAny>> {
        let client = self.client.clone();
        let result = block_on(py, async move {
            client.delete_raw(&path).await.into_py_result()
        })?;
        Ok(json_to_py(py, result))
    }
}

/// Convert serde_json::Value to Python object
pub fn json_to_py(py: Python<'_>, value: serde_json::Value) -> Py<PyAny> {
    match value {
        serde_json::Value::Null => py.None(),
        serde_json::Value::Bool(b) => b.into_pyobject(py).unwrap().to_owned().into_any().unbind(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_pyobject(py).unwrap().to_owned().into_any().unbind()
            } else if let Some(f) = n.as_f64() {
                f.into_pyobject(py).unwrap().to_owned().into_any().unbind()
            } else {
                py.None()
            }
        }
        serde_json::Value::String(s) => s.into_pyobject(py).unwrap().to_owned().into_any().unbind(),
        serde_json::Value::Array(arr) => {
            let list = PyList::new(py, arr.into_iter().map(|v| json_to_py(py, v))).unwrap();
            list.into_any().unbind()
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_to_py(py, v)).unwrap();
            }
            dict.into_any().unbind()
        }
    }
}

/// Convert Python object to serde_json::Value
pub fn py_to_json(py: Python<'_>, obj: Py<PyAny>) -> PyResult<serde_json::Value> {
    let obj = obj.bind(py);

    if obj.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(serde_json::Value::Number(i.into()))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(serde_json::json!(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(serde_json::Value::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let arr: PyResult<Vec<serde_json::Value>> = list
            .iter()
            .map(|item| py_to_json(py, item.unbind()))
            .collect();
        Ok(serde_json::Value::Array(arr?))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, py_to_json(py, v.unbind())?);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Cannot convert Python object to JSON",
        ))
    }
}
