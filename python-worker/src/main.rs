#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::prelude::*;

use rocket_contrib::json::Json;
use std::process::Command;
use smart::{SmartSubmission, SmartResponse};

fn clear_environment() {
    // Delete the files "environment/solution.py" and "environment/test.py"
    std::fs::remove_file("environment/solution.py").unwrap();
    std::fs::remove_file("environment/test.py").unwrap();
}

#[post("/evaluate", format = "application/json", data = "<submission>")]
fn evaluate(submission: Json<SmartSubmission>) -> String {
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
