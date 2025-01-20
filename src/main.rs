#![feature(iter_advance_by)]
#![feature(str_split_whitespace_remainder)]
use std::{
    env::{self, current_exe},
    fmt::Error,
    ops::Range,
    path::PathBuf,
    process::{Command, Output},
    str::from_utf8,
};

use cmd_lib::{run_cmd, run_fun};

#[derive(PartialEq)]
enum NetworkCondition {
    Good,
    Bad,
}

struct PythonDevice {
    network_condition: NetworkCondition,
}
impl PythonDevice {
    fn new() -> PythonDevice {
        PythonDevice {
            network_condition: NetworkCondition::Good,
        }
    }
}

impl PythonDevice {
    fn get_pdm() -> PathBuf {
        let APPDATA: PathBuf = env::var("APPDATA").unwrap().into();
        APPDATA.join("Python/Scripts/pdm.exe")
    }
    fn set_network_condition_bad(&mut self) {
        self.network_condition = NetworkCondition::Bad;
        println!("network condition is bad")
    }
    fn update_pdm(&mut self) -> Result<(), ()> {
        let pdm = Self::get_pdm();
        if run_fun! {$pdm}.is_ok() {
            if self.network_condition == NetworkCondition::Bad {
                return Ok(());
            }
            println!("update pdm");
            if run_cmd! {$pdm self update}.is_ok() {
                Ok(())
            } else {
                self.set_network_condition_bad();
                Ok(())
            }
        } else {
            if self.network_condition == NetworkCondition::Bad {
                return Err(());
            }
            println!("install pdm");
            if run_cmd!{powershell -ExecutionPolicy ByPass -c "irm https://pdm-project.org/install-pdm.py | py -"}.is_ok(){
                Ok(())
            } else {
                self.set_network_condition_bad();
                Err(())
            }
        }
    }
    fn update(&mut self) -> Result<(), ()> {
        if !cfg!(target_os = "windows") {
            panic!("current OS is not surported")
        };
        self.update_pdm()?;
        Ok(())
    }
}
fn main() {
    let mut python_device = PythonDevice::new();
    if let Err(_) = python_device.update() {
        return;
    }
}
