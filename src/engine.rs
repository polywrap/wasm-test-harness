use std::collections::HashMap;
use std::{fs, io};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::string::FromUtf8Error;
use crate::{Results};
use crate::generator::{Generate};
use serde::{Deserialize, Serialize};
use crate::error::{BuildError, TestError, GenerateError, ExecutionError};
use crate::error::BuildError::BuildExecutionError;
use crate::error::TestError::TestExecutionError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Engine {
    pub destination_path: String,
    pub source_path: String,
    pub feature: String,
    pub implementation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Executor {
    Build,
    Run,
    Generate
}

impl Engine {
    pub fn new() -> Self {
        Self {
            destination_path: String::new(),
            source_path: String::new(),
            feature: String::new(),
            implementation: String::new(),
        }
    }

    pub fn set_case(
        &mut self,
        destination_path: &Path,
        source_path: &Path,
        feature: String,
        implementation: String,
    )  {
        self.destination_path = String::from(destination_path.to_str().unwrap());
        self.source_path = String::from(source_path.to_str().unwrap());
        self.feature = feature;
        self.implementation = implementation;
    }

    pub fn execute(&mut self, action: Executor) -> Result<(), ExecutionError> {
        let wrapper_path = Path::new(
            &self.destination_path.as_str()
        ).join(&self.feature).join("implementations");
        let dir = &wrapper_path.join(&self.implementation);
        match action {
            Executor::Generate => {
                let destination_path = Path::new(self.destination_path.as_str());
                let generate = Generate::new(
                    destination_path.canonicalize().unwrap().as_path(),
                    Path::new(self.source_path.as_str()),
                )?;

                generate.project(
                    self.feature.as_str(),
                    self.implementation.as_str()
                )?;
            }
            Executor::Build => {
                if !self.implementation.is_empty() {
                    self.build(dir)?;
                } else {
                    for implementation in read_dir(&wrapper_path).unwrap() {
                        self.build(&implementation.unwrap().path())?;
                    }
                }
            }
            Executor::Run => {
                if !self.implementation.is_empty() {
                    self.run(dir)?;
                } else {
                    for implementation in read_dir(&wrapper_path).unwrap() {
                        self.run(&implementation.unwrap().path())?;
                    }
                }
                Results::show()?;
            }
        };
        Ok(())
    }

    pub fn build(&self, dir: &PathBuf) -> Result<(), BuildError> {
        println!(
            "Building implementation: {} in test case {}",
            &self.implementation,
            self.feature
        );
        let mut build = Command::new("node");
        build.current_dir(dir.canonicalize()?);
        build.arg("../../../../../monorepo/packages/cli/bin/polywrap").arg("build").arg("-v");

        match build.output() {
            Ok(t) => {
                let error = String::from_utf8(t.stderr)?;
                if !error.is_empty() {
                    return Err(BuildExecutionError("Build command has failed".to_string()))
                }
                // let message = String::from_utf8(t.stdout)?;
                t.status.success()
            }
            Err(e) => {
                dbg!(e);
                false
            }
        };
        Ok(())
    }

    pub fn run(&self, dir: &PathBuf) -> Result<(), TestError> {
        let mut run = Command::new("node");
        run.current_dir(dir.canonicalize()?);
        run.arg("../../../../../monorepo/packages/cli/bin/polywrap").arg("run")
            .arg("-m").arg("../../polywrap.test.yaml")
            .arg("-o").arg("./output.json");

        let custom_config = dir.join("../../client-config.ts").exists();
        if custom_config {
            run.arg("-c").arg("../../client-config.ts");
        }

        match run.output() {
            Ok(t) => {
                let error = String::from_utf8(t.stderr)?;
                if !error.is_empty() {
                    return Err(TestExecutionError("Run command has failed".to_string()))
                }
                // let message = String::from_utf8(t.stdout)?;

                let impl_name = dir.file_name().unwrap().to_str().unwrap();
                let results_dir = dir.join("output.json");
                let summary = Results::process(results_dir)?;

                let info_path = Path::new(self.destination_path.as_str())
                    .join("..")
                    .join("results.json");
                let feature_name = &self.feature;
                match fs::read(&info_path) {
                    Ok(f) => {
                        let result_str = String::from_utf8_lossy(&f).parse::<String>().unwrap();
                        let mut results: Results = serde_json::from_str(result_str.as_str()).unwrap();
                        results.info.entry(impl_name.to_string()).or_default().insert(feature_name.to_string(), summary);
                        let results_file = fs::OpenOptions::new()
                            .write(true)
                            .open(&info_path)
                            .unwrap();
                        serde_json::to_writer_pretty(results_file, &results).unwrap();
                    }
                    Err(_) => {
                        let mut results = Results::new();
                        let summaries = HashMap::from([
                            (feature_name.to_string(), summary)
                        ]);
                        results.info.insert(impl_name.to_string(), summaries);
                        let results_file = fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(&info_path)
                            .unwrap();
                        serde_json::to_writer_pretty(results_file, &results).unwrap();
                    }
                };
            }
            Err(e) => {
                dbg!(e);
            }
        };
        Ok(())
    }
}
