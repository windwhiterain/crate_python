use std::{
    env,
    ffi::{CStr, CString},
    fs,
    str::FromStr,
};

use pyo3::{
    PyResult, Python,
    types::{PyAnyMethods, PyModule},
};
fn main() {
    println!("{}", env::current_exe().unwrap().display());
    let activate_this_dir = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("python_project/.venv/Scripts/activate_this.py");
    Python::with_gil(|py| {
        let runpy = py.import("runpy").unwrap();
        runpy
            .call_method1("run_path", (activate_this_dir,))
            .unwrap();
        let pylib = py.import("pylib").unwrap();
    });
}
