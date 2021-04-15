#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use std::{thread, time};

lazy_static! {
    static ref WORKER_IDLING: Mutex<Vec<Worker>> = Mutex::new(vec![]);
    static ref WORKER_RUNNING: Mutex<Vec<Worker>> = Mutex::new(vec![]);
    static ref WORKER_NO_CONNECTION: Mutex<Vec<Worker>> = Mutex::new(vec![]);
}

#[derive(Debug, Clone)]
struct Worker {
    address: String,
}

#[post("/evaluate/<lang>", format = "application/json", data = "<submission>")]
fn evaluate(lang: String, submission: String) -> String {
    match lang.to_lowercase().as_ref() {
        "python" => evaluate_python(submission),
        _ => format!("Could not find test suite for language: {}", lang),
    }
}

fn evaluate_python(submission: String) -> String {
    let client_address = WORKER_IDLING.lock().unwrap().pop();
    if let Some(client_address) = client_address {
        WORKER_RUNNING.lock().unwrap().push(client_address.clone());
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(client_address.clone().address + "/evaluate")
            .body(submission.clone())
            .send();

        if res.is_err() {
            println!(
                "Connection failed to worker: {}",
                client_address.clone().address
            );
            WORKER_NO_CONNECTION
                .lock()
                .unwrap()
                .push(client_address.clone());
            WORKER_RUNNING
                .lock()
                .unwrap()
                .retain(|x| x.address != client_address.clone().address);
            return evaluate_python(submission);
        }

        let res = res.unwrap();
        WORKER_RUNNING
            .lock()
            .unwrap()
            .retain(|x| x.address != client_address.clone().address);

        WORKER_IDLING.lock().unwrap().push(client_address.clone());
        if res.status().is_success() {
            return format!("{}", res.text().unwrap());
        } else {
            return format!("404");
        }
    } else {
        let ten_millis = time::Duration::from_millis(500);
        thread::sleep(ten_millis);
        return evaluate_python(submission);
    }
}

fn main() {
    WORKER_IDLING.lock().unwrap().push(Worker {
        address: String::from("http://localhost:8081"),
    });
    // WORKER_IDLING.lock().unwrap().push(Worker {
    //     address: String::from("http://localhost:8082"),
    // });
    // WORKER_IDLING.lock().unwrap().push(Worker {
    //     address: String::from("http://localhost:8083"),
    // });
    // WORKER_IDLING.lock().unwrap().push(Worker {
    //     address: String::from("http://localhost:8084"),
    // });
    // WORKER_IDLING.lock().unwrap().push(Worker {
    //     address: String::from("http://localhost:8085"),
    // });

    rocket::ignite().mount("/", routes![evaluate]).launch();
}
