use std::{
    io::{Cursor, Write},
    path::PathBuf,
};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::{PyBytes, PyIterator, PyList, PyTuple},
};
use pyo3_file::PyFileLikeObject;
use pythonize::{Depythonizer, Pythonizer};
use rmp_serde::{decode::ReadSlice, Deserializer, Serializer};
use serde_transcode::transcode;

struct MpkzIterator<'py, R> {
    py: Python<'py>,
    de: Deserializer<R>,
}
impl<'py, R> MpkzIterator<'py, R> {
    pub fn new(py: Python<'py>, de: Deserializer<R>) -> Self {
        Self { py, de }
    }
}

impl<'py, R: ReadSlice<'py>> Iterator for MpkzIterator<'py, R> {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Self::Item> {
        let ser = Pythonizer::new(self.py);
        transcode(&mut self.de, ser).ok()
    }
}

fn decode_all<'py, R: ReadSlice<'py>>(py: Python<'py>, de: Deserializer<R>) -> PyResult<Py<PyAny>> {
    let rows: Vec<_> = MpkzIterator::new(py, de).collect();
    if rows.len() == 1 {
        Ok(rows.into_iter().next().unwrap())
    } else {
        Ok(rows.into_py(py))
    }
}

fn encode_all<'py, W: Write>(
    obj: &'py PyAny,
    mut ser: Serializer<zstd::Encoder<'py, W>>,
) -> PyResult<W> {
    if let Ok(list) = obj.downcast::<PyList>() {
        for row in list.iter() {
            let mut de = Depythonizer::from_object(row);
            transcode(&mut de, &mut ser).map_err(|e| PyValueError::new_err((e.to_string(),)))?;
        }
    } else {
        let mut de = Depythonizer::from_object(obj);
        transcode(&mut de, &mut ser).map_err(|e| PyValueError::new_err((e.to_string(),)))?;
    }
    Ok(ser.into_inner().finish()?)
}

#[pyfunction]
fn load<'py>(py: Python<'py>, fp: Py<PyAny>) -> PyResult<Py<PyAny>> {
    let file = PyFileLikeObject::with_requirements(fp, true, false, false, false)?;
    let decoder = zstd::Decoder::new(file)?;
    let de = Deserializer::new(decoder);
    decode_all(py, de)
}

#[pyfunction]
fn loadb<'py>(py: Python<'py>, bytes: &'py PyBytes) -> PyResult<Py<PyAny>> {
    let decoder = zstd::Decoder::new(bytes.as_bytes())?;
    let de = Deserializer::new(decoder);
    decode_all(py, de)
}

#[pyfunction]
#[pyo3(signature = (obj, fp, *, level=8))]
fn dump(obj: &PyAny, fp: Py<PyAny>, level: i32) -> PyResult<()> {
    let file = PyFileLikeObject::with_requirements(fp, false, true, false, false)?;
    let encoder = zstd::Encoder::new(file, level)?;
    let ser = Serializer::new(encoder);
    encode_all(obj, ser)?;
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (obj, *, level=8))]
fn dumpb<'py>(py: Python<'py>, obj: &'py PyAny, level: i32) -> PyResult<&'py PyBytes> {
    let cursor = Cursor::new(Vec::new());
    let encoder = zstd::Encoder::new(cursor, level)?;
    let ser = Serializer::new(encoder);
    let bytes = encode_all(obj, ser)?.into_inner();
    Ok(PyBytes::new(py, &bytes))
}

#[pyfunction]
fn open<'py>(py: Python<'py>, filename: PathBuf) -> PyResult<PyObject> {
    let file = std::fs::File::open(filename)?;
    let decoder = zstd::Decoder::new(file)?;
    let mut de = Deserializer::new(decoder);
    Ok(MpkzReader::new(
        py,
        Box::new(move |py: Python| {
            let ser = Pythonizer::new(py);
            transcode(&mut de, ser).ok()
        }),
    ))
}

#[pyfunction]
fn openb<'py>(py: Python<'py>, data: Vec<u8>) -> PyResult<PyObject> {
    let cursor = Cursor::new(data);
    let decoder = zstd::Decoder::new(cursor)?;
    let mut de = Deserializer::new(decoder);
    Ok(MpkzReader::new(
        py,
        Box::new(move |py: Python| {
            let ser = Pythonizer::new(py);
            transcode(&mut de, ser).ok()
        }),
    ))
}

#[pyfunction]
#[pyo3(signature = (filename, *, level=8))]
fn create(py: Python, filename: PathBuf, level: i32) -> PyResult<PyObject> {
    let file = std::fs::File::create(filename)?;
    let encoder = zstd::Encoder::new(file, level)?;
    let ser = Serializer::new(encoder);
    let inner = InternalWriter::new(ser);
    let writer = MpkzWriter { inner };
    Ok(writer.into_py(py))
}

#[pyclass(unsendable)]
struct MpkzReader {
    func: Box<dyn FnMut(Python) -> Option<PyObject>>,
}
impl MpkzReader {
    fn new(py: Python, func: Box<dyn FnMut(Python) -> Option<PyObject>>) -> PyObject {
        let func = Box::new(func);
        Self { func }.into_py(py)
    }
}
#[pymethods]
impl MpkzReader {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        let py = slf.py();
        (slf.func)(py)
    }
}

trait AnyWriter {
    fn _write(&mut self, obj: &PyAny) -> PyResult<()>;
    fn _finish(&mut self) -> PyResult<()>;
}
struct InternalWriter<'a, W: Write> {
    ser: Option<Serializer<zstd::Encoder<'a, W>>>,
}
impl<'a, W: Write> InternalWriter<'a, W> {
    fn new(ser: Serializer<zstd::Encoder<'a, W>>) -> Box<Self> {
        let ser = Some(ser);
        Box::new(Self { ser })
    }
}
impl<'a, W: Write> AnyWriter for InternalWriter<'a, W> {
    fn _write(&mut self, obj: &PyAny) -> PyResult<()> {
        if let Some(ser) = &mut self.ser {
            let mut de = Depythonizer::from_object(obj);
            transcode(&mut de, ser).map_err(|e| PyValueError::new_err((e.to_string(),)))?;
        } else {
            return Err(PyRuntimeError::new_err(("Writer has already finished",)));
        }
        Ok(())
    }

    fn _finish(&mut self) -> PyResult<()> {
        if let Some(ser) = self.ser.take() {
            ser.into_inner().finish()?;
        }
        Ok(())
    }
}

#[pyclass(unsendable)]
struct MpkzWriter {
    inner: Box<dyn AnyWriter>,
}
#[pymethods]
impl MpkzWriter {
    fn append(&mut self, obj: &PyAny) -> PyResult<()> {
        self.inner._write(obj)
    }

    fn extend(&mut self, obj: &PyAny) -> PyResult<()> {
        let iter = PyIterator::from_object(obj)?;
        for item in iter {
            self.inner._write(item?)?;
        }
        Ok(())
    }

    fn finish(&mut self) -> PyResult<()> {
        self.inner._finish()
    }

    fn __enter__<'p>(slf: PyRef<'p, Self>, _py: Python<'p>) -> PyResult<PyRef<'p, Self>> {
        Ok(slf)
    }

    #[pyo3(signature = (*_args))]
    fn __exit__(&mut self, _args: &PyTuple) -> PyResult<()> {
        self.finish()
    }
}

impl Drop for MpkzWriter {
    fn drop(&mut self) {
        _ = self.finish();
    }
}

#[pymodule]
fn mpkz(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MpkzReader>()?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(loadb, m)?)?;
    m.add_function(wrap_pyfunction!(dump, m)?)?;
    m.add_function(wrap_pyfunction!(dumpb, m)?)?;
    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add_function(wrap_pyfunction!(openb, m)?)?;
    m.add_function(wrap_pyfunction!(create, m)?)?;
    Ok(())
}
