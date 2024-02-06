use std::io::Cursor;

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyDict, PyTuple},
};
use pyo3_file::PyFileLikeObject;
use pythonize::{Depythonizer, Pythonizer};
use rmp_serde::{Deserializer, Serializer};
use serde_transcode::transcode;

/// note to ipython: please use the pyi file, thanks
#[pyfunction]
fn load<'py>(py: Python<'py>, fp: Py<PyAny>) -> PyResult<Py<PyAny>> {
    let file = PyFileLikeObject::with_requirements(fp, true, false, false, false)?;
    let decoder = zstd::Decoder::new(file)?;
    let mut de = Deserializer::new(decoder);
    let ser = Pythonizer::new(py);
    let x = transcode(&mut de, ser).map_err(|e| PyValueError::new_err((e.to_string(),)))?;
    Ok(x)
}

/// note to ipython: please use the pyi file, thanks
#[pyfunction]
fn loads<'py>(py: Python<'py>, bytes: &'py PyBytes) -> PyResult<Py<PyAny>> {
    let decoder = zstd::Decoder::new(bytes.as_bytes())?;
    let mut de = Deserializer::new(decoder);
    let ser = Pythonizer::new(py);
    let x = transcode(&mut de, ser).map_err(|e| PyValueError::new_err((e.to_string(),)))?;
    Ok(x)
}

/// note to ipython: please use the pyi file, thanks
#[pyfunction]
#[pyo3(signature = (obj, fp, *, level=8))]
fn dump(obj: &PyAny, fp: Py<PyAny>, level: i32) -> PyResult<()> {
    let file = PyFileLikeObject::with_requirements(fp, false, true, false, false)?;
    let encoder = zstd::Encoder::new(file, level)?;
    let mut de = Depythonizer::from_object(obj);
    let mut ser = Serializer::new(encoder);
    transcode(&mut de, &mut ser).map_err(|e| PyValueError::new_err((e.to_string(),)))?;
    ser.into_inner().finish()?;
    Ok(())
}

/// note to ipython: please use the pyi file, thanks
#[pyfunction]
#[pyo3(signature = (obj, *, level=8))]
fn dumps<'py>(py: Python<'py>, obj: &'py PyAny, level: Option<i32>) -> PyResult<&'py PyBytes> {
    let level = level.unwrap_or(8);
    let cursor = Cursor::new(Vec::new());
    let encoder = zstd::Encoder::new(cursor, level)?;
    let mut de = Depythonizer::from_object(obj);
    let mut ser = Serializer::new(encoder);
    transcode(&mut de, &mut ser).map_err(|e| PyValueError::new_err((e.to_string(),)))?;
    let bytes = ser.into_inner().finish()?.into_inner();
    Ok(PyBytes::new(py, &bytes))
}

/// note to ipython: please use the pyi file, thanks
#[pymodule]
fn mpkz(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    m.add_function(wrap_pyfunction!(dump, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    let child = PyModule::new(py, "mpkz.jsonz")?;
    jsonz(py, child)?;
    m.add("jsonz", child)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("supermodule.submodule", child)?;
    Ok(())
}
