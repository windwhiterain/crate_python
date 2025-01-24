use crate_python;
use pyo3::{Py, PyAny, Python, types::PyAnyMethods};
fn main() {
    crate_python::init();
    let mut a: Option<Py<PyAny>> = None;
    Python::with_gil(|py| {
        let pylib = py.import("pylib").unwrap();
        let a_type = pylib.getattr("A").unwrap();
        let a_instance = a_type.call0().unwrap();
        a = Some(a_instance.unbind());
    });
    Python::with_gil(|py| {
        let binding = a.unwrap();
        let a_instance = binding.bind(py);
        a_instance.call_method0("a").unwrap();
    });
}
