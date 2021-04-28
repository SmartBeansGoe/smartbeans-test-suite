#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::prelude::*;

use rocket_contrib::json::Json;
use serde::Deserialize;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
enum SmartResult {
    Unknown,
    EvaluationError,
    ContainerError,
    Success,
    Failed,
    TimeOut,
}

#[derive(Debug, Deserialize)]
struct Submission {
    source: String,
    test: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SmartResponse {
    result_type: SmartResult,
    runs: u32,
    errors: u32,
    failures: u32,
    feedback: String,
}

impl SmartResponse {
    // Updates the result_type.
    fn update(mut self) -> Self {
        if self.errors == 0 && self.failures == 0 {
            self.result_type = SmartResult::Success;
        } else {
            self.result_type = SmartResult::Failed
        }
        self
    }
}

#[post("/evaluate", format = "application/json", data = "<submission>")]
fn evaluate(submission: Json<Submission>) -> String {
    let mut file = File::create("environment/solution.py").unwrap();
    file.write_all(submission.source.as_bytes()); // Container defekt error einbauen, wenn keine Datei erstellt werden kann.
    let mut file = File::create("environment/test.py").unwrap();
    file.write_all(submission.test.as_bytes());

    let c = Command::new("python3")
        .current_dir("environment/")
        .args(&["test.py"])
        .output()
        .unwrap();

    let res: SmartResponse = serde_json::from_slice(&c.stdout).unwrap_or(SmartResponse {
        result_type: SmartResult::EvaluationError,
        runs: 0,
        errors: 0,
        failures: 0,
        feedback: "EVALUATION_ERROR means that there is a server side error.\n\"The test script is corrupted.\"".to_string(),    
    }).update();

    // Delete the files "environment/solution.py" and "environment/test.py"
    std::fs::remove_file("environment/solution.py").unwrap();
    std::fs::remove_file("environment/test.py").unwrap();

    serde_json::to_string(&res).unwrap()
}

fn main() {
    rocket::ignite().mount("/", routes![evaluate]).launch();
}
