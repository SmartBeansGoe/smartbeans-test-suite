#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::prelude::*;

use rocket_contrib::json::Json;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::PartialEq;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum SmartResult {
    Undefined,
    ParsingError,
    UnknownError,
    EvaluationError,
    ContainerError,
    Success,
    Failed,
    TimedOut,
}

#[derive(Debug, Deserialize)]
struct Submission {
    source_code: String,
    tests: String,
    timeout: String,
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
        if self.result_type != SmartResult::EvaluationError
            && self.result_type != SmartResult::TimedOut
            && self.result_type != SmartResult::UnknownError
        {
            if self.errors == 0 && self.failures == 0 {
                self.result_type = SmartResult::Success;
            } else {
                self.result_type = SmartResult::Failed
            }
        }
        self
    }

    // Generates the SmartResponse from json output of the test run.
    // Expected is a JSON which contains all entries of SmartResponse.
    // The result_type could be set as unknown in the test.
    fn from_output(output: std::process::Output) -> SmartResponse {
        let res: SmartResponse = serde_json::from_slice(&output.stdout).unwrap_or({
            let result_type;
            let feedback;
            match output.status.code() {
                Some(124) => {
                    result_type = SmartResult::TimedOut;
                    feedback ="TimedOut: The test takes to long.".to_string();
                },
                Some(1) => {
                    result_type = SmartResult::EvaluationError;
                    feedback = "EVALUATION_ERROR means that there is a server side error.\n\"The test script is corrupted.\"".to_string();
                }
                _ => {
                    if output.stdout.len() > 0 {
                        result_type = SmartResult::ParsingError;
                        feedback = "ParsingError: The output of the test script is not as expected.".to_string();    
                    }
                    else {
                        result_type = SmartResult::UnknownError;
                        feedback = "UnknownError".to_string();
                    }
                }
            };
            SmartResponse {
                result_type: result_type,
                runs: 0,
                errors: 0,
                failures: 0,
                feedback: feedback,            
            }
        }).update();
        res
    }
}

fn clear_environment() {
    // Delete the files "environment/solution.py" and "environment/test.py"
    std::fs::remove_file("environment/solution.py").unwrap();
    std::fs::remove_file("environment/test.py").unwrap();
}

#[post("/evaluate", format = "application/json", data = "<submission>")]
fn evaluate(submission: Json<Submission>) -> String {
    let mut file = File::create("environment/solution.py").unwrap();
    file.write_all(submission.source_code.as_bytes()); // TODO: ContainerError if user has no rights to write.
    let mut file = File::create("environment/test.py").unwrap();
    file.write_all(submission.tests.as_bytes());

    let c = Command::new("timeout")
        .current_dir("environment/")
        .args(&[submission.timeout.as_str(), "python3", "test.py"])
        .output()
        .unwrap();

    println!("{:?}", c);

    let res = SmartResponse::from_output(c);

    clear_environment();

    serde_json::to_string(&res).unwrap()
}

fn main() {
    rocket::ignite().mount("/", routes![evaluate]).launch();
}
