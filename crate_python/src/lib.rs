#![feature(iter_advance_by)]
#![feature(str_split_whitespace_remainder)]
#![feature(os_str_display)]

pub mod pyproject;

use std::{env, fs, path::PathBuf};

use cmd_lib::{run_cmd, run_fun};

use build_print::{println, *};
use pyproject::PyProject;

#[derive(PartialEq)]
enum NetworkCondition {
    Good,
    Bad,
}

pub struct Builder {
    network_condition: NetworkCondition,
}
impl Builder {
    pub fn new() -> Builder {
        Builder {
            network_condition: NetworkCondition::Good,
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
        self.bin_dir().join("python_project")
    }
    fn set_network_condition_bad(&mut self) {
        self.network_condition = NetworkCondition::Bad;
        warn!("network condition is bad")
    }
    fn update_pdm(&mut self) -> Result<(), ()> {
        let pdm = self.pdm_dir();
        if run_fun! {${pdm}}.is_ok() {
            if self.network_condition == NetworkCondition::Bad {
                return Ok(());
            }
            if run_cmd! {${pdm} self update}.is_ok() {
                println!("pdm self update");
                Ok(())
            } else {
                self.set_network_condition_bad();
                Ok(())
            }
        } else {
            if self.network_condition == NetworkCondition::Bad {
                return Err(());
            }
            if run_cmd!{powershell -ExecutionPolicy ByPass -c "irm https://pdm-project.org/install-pdm.py | py -"}.is_ok(){
                println!("install pdm");
                Ok(())
            } else {
                self.set_network_condition_bad();
                Err(())
            }
        }
    }
    fn update_python_project(&mut self) -> Result<(), ()> {
        let python_project = self.python_project_dir();
        match run_cmd! {
            cd ${python_project};
            pdm update;
        } {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e);
                Err(())
            }
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
}

pub fn build_bin(libs: &Vec<Config>) {
    let mut device = Builder::new();
    let version = pyo3_build_config::get().version;
    let python_project_dir = device.python_project_dir();
    let pyproject_toml_dir = python_project_dir.join("pyproject.toml");
    let pdm_lock_dir = python_project_dir.join("pdm.lock");
    let _ = fs::create_dir(python_project_dir);
    let _ = fs::remove_file(pdm_lock_dir);
    let mut pyproject = PyProject::default();
    pyproject.project.requires_python = Some(format!("=={}.{}.*", version.major, version.minor));
    for lib in libs {
        let name = lib.dir.file_name().unwrap();
        let python = lib.dir.join("python");
        if python.exists() {
            pyproject.project.dependencies.push(format!(
                "{} @ file:///{}",
                name.display(),
                python.display()
            ));
        }
    }
    fs::write(pyproject_toml_dir, toml::to_string(&pyproject).unwrap()).unwrap();
    device.update_pdm().unwrap();
    device.update_python_project().unwrap();
}

#[macro_export]
macro_rules! config_lib {
    ( $( $x:ident ),* ) => {
        pub fn crate_python_configs()->Vec<crate_python::Config>
        {
            let mut ret:Vec<crate_python::Config> = vec![];
            $(
                ret.append(&mut $x::crate_python_configs());
            )*
            ret.push(crate_python::Config{dir:core::env!("CARGO_MANIFEST_DIR").into()});
            ret
        }
        fn crate_python_build_bin()
        {
            crate_python::build_bin(&crate_python_configs());
        }
    };
}
