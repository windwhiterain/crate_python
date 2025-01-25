#![feature(iter_advance_by)]
#![feature(str_split_whitespace_remainder)]
#![feature(os_str_display)]

pub mod pyproject;
mod utils;

use std::{env, fs, path::PathBuf};

use cmd_lib::{run_cmd, run_fun};

use pyo3::{Python, types::PyAnyMethods};
use pyproject::PyProject;
use toml::Table;

#[derive(PartialEq)]
enum NetworkCondition {
    Good,
    Bad,
}

pub struct Device {
    network_condition: NetworkCondition,
}
impl Device {
    fn new() -> Device {
        Device {
            network_condition: NetworkCondition::Good,
        }
    }
    fn is_dev(&mut self) -> bool {
        match option_env!("CRATE_PYTHON_DEV") {
            Some(crate_python_dev) => crate_python_dev != "0",
            None => false,
        }
    }
    fn python_project_name(&mut self) -> &str {
        match self.is_dev() {
            true => "python_project_dev",
            false => "python_project",
        }
    }
    fn pdm_dir(&mut self) -> PathBuf {
        let appdata: PathBuf = env::var("APPDATA").unwrap().into();
        appdata.join("Python/Scripts/pdm.exe")
    }
    fn bin_dir(&mut self) -> PathBuf {
        let out_dir: PathBuf = env::var("OUT_DIR").unwrap().into();
        out_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .into()
    }
    fn python_project_dir(&mut self) -> PathBuf {
        self.bin_dir().join(self.python_project_name())
    }
    fn set_network_condition_bad(&mut self) {
        self.network_condition = NetworkCondition::Bad;
        build_print::warn!("assume that network condition is bad")
    }
    fn update_pdm(&mut self) {
        let pdm = self.pdm_dir();
        if run_fun! {${pdm}}.is_ok() {
            run_cmd! {
                ${pdm} config install.cache True
            }
            .unwrap();
            if self.network_condition == NetworkCondition::Bad {
                return;
            }
            match run_cmd! {${pdm} self update} {
                Ok(_) => {
                    build_print::info!("pdm self update");
                }
                Err(e) => {
                    build_print::warn!("{}", e);
                    self.set_network_condition_bad();
                }
            };
        } else {
            if self.network_condition == NetworkCondition::Bad {
                panic!();
            }
            match run_cmd! {powershell -ExecutionPolicy ByPass -c "irm https://pdm-project.org/install-pdm.py | py -"}
            {
                Ok(_) => {
                    build_print::info!("install pdm");
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
    fn update_python_project(&mut self) {
        let python_project = self.python_project_dir();
        let pdm = self.pdm_dir();
        match if self.is_dev() {
            run_fun! {
                cd ${python_project};
                ${pdm} update;
            }
        } else {
            run_fun! {
                cd ${python_project};
                pdm config install.cache False;
                ${pdm} update --no-editable;
                pdm config install.cache True;
            }
        } {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct Config {
    pub has_python: bool,
    pub dir: PathBuf,
}

pub fn build_bin<'a>(libs: &mut impl Iterator<Item = &'a Config>) {
    println!("cargo::rerun-if-env-changed=CRATE_PYTHON_DEV");

    let mut device = Device::new();
    let pyo3_config = pyo3_build_config::get();
    let version = pyo3_config.version;
    let python_exe_path: PathBuf = pyo3_config.executable.as_ref().unwrap().into();
    let python_exe_dir = python_exe_path.parent().unwrap();
    let python_dll_name = format!("python{}{}.dll", &version.major, &version.minor);
    let python_project_dir = device.python_project_dir();
    let pyproject_toml_path = python_project_dir.join("pyproject.toml");
    let pdm_lock_path = python_project_dir.join("pdm.lock");
    let _ = fs::create_dir(&python_project_dir);
    let _ = fs::remove_file(pdm_lock_path);
    if !device.is_dev() {
        let venv_dir = python_project_dir.join(".venv");
        let _ = fs::remove_dir_all(venv_dir);
    }
    let mut pyproject = PyProject::default();
    pyproject.project.requires_python = Some(format!("=={}.{}.*", version.major, version.minor));
    for lib in libs {
        if lib.has_python {
            let python_dir = lib.dir.join("python");
            let pyproject_toml_path = python_dir.join("pyproject.toml");
            if !pyproject_toml_path.exists() {
                panic!(
                    "python crate [{}] is configed to has a python project, but pyproject.toml at [{}] is not found",
                    lib.dir.display(),
                    pyproject_toml_path.display()
                );
            }
            println!(
                "cargo::rerun-if-changed={}",
                if device.is_dev() {
                    &pyproject_toml_path
                } else {
                    &python_dir
                }
                .display()
            );
            let lib_pyproject: Table =
                toml::from_str(&fs::read_to_string(pyproject_toml_path).unwrap()).unwrap();
            let name = lib_pyproject
                .get("project")
                .unwrap()
                .as_table()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap();

            match device.is_dev() {
                false => pyproject.project.dependencies.insert(format!(
                    "{} @ file:///{}",
                    name,
                    python_dir.display()
                )),
                true => pyproject.tool.pdm.dev_dependencies.dev.insert(format!(
                    "-e file:///{}#egg={}",
                    python_dir.display(),
                    name
                )),
            };
        }
    }
    fs::write(pyproject_toml_path, toml::to_string(&pyproject).unwrap()).unwrap();
    device.update_pdm();
    device.update_python_project();
    if !device.is_dev() {
        fs::copy(
            python_exe_dir.join(&python_dll_name),
            device.bin_dir().join(&python_dll_name),
        )
        .unwrap();
        let _ = fs::remove_dir_all(device.bin_dir().join("Lib"));
        utils::copy_dir_all(python_exe_dir.join("Lib"), device.bin_dir().join("Lib")).unwrap();
    }
}

#[macro_export]
macro_rules! config {
    ($has_python:expr, $( $dependency:ident ),* ) => {
        pub fn crate_python_configs()->std::collections::HashSet<crate_python::Config>
        {
            let mut ret:std::collections::HashSet<crate_python::Config> = Default::default();
            $(
                ret.extend(&mut $dependency::crate_python_configs().into_iter());
            )*
            ret.insert(crate_python::Config{has_python:$has_python,dir:core::env!("CARGO_MANIFEST_DIR").into()});
            ret
        }
        fn crate_python_build_bin()
        {
            crate_python::build_bin(&mut crate_python_configs().iter());
        }
    };
}

pub fn init() {
    pyo3::prepare_freethreaded_python();
    let mut deivice = Device::new();
    let exe_dir: PathBuf = env::current_exe().unwrap().parent().unwrap().into();
    let activate_this_dir = exe_dir
        .join(deivice.python_project_name())
        .join(".venv/Scripts/activate_this.py");
    let lib_dir = exe_dir.join("Lib");
    Python::with_gil(|py| {
        let sys = py.import("sys").unwrap();
        let path = sys.getattr("path").unwrap();
        path.call_method1("append", (lib_dir,)).unwrap();
        let runpy = py.import("runpy").unwrap();
        runpy
            .call_method1("run_path", (activate_this_dir,))
            .unwrap();
    });
}
