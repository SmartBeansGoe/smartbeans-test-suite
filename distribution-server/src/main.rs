#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref WORKER_IDLING: Mutex<Vec<worker::Worker>> = Mutex::new(vec![]);
    static ref WORKER_RUNNING: Mutex<Vec<worker::Worker>> = Mutex::new(vec![]);
    static ref WORKER_NO_CONNECTION: Mutex<Vec<worker::Worker>> = Mutex::new(vec![]);
}

mod worker;

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
            .post(client_address.clone().ip().to_string() + "/evaluate")
            .body(submission.clone())
            .send();

        if res.is_err() {
            println!(
                "Connection failed to worker: {}",
                client_address.clone().ip()
            );
            WORKER_NO_CONNECTION
                .lock()
                .unwrap()
                .push(client_address.clone());
            WORKER_RUNNING
                .lock()
                .unwrap()
                .retain(|x| x.ip() != client_address.clone().ip());
            return evaluate_python(submission);
        }

        let res = res.unwrap();
        WORKER_RUNNING
            .lock()
            .unwrap()
            .retain(|x| x.ip() != client_address.clone().ip());

        WORKER_IDLING.lock().unwrap().push(client_address.clone());
        if res.status().is_success() {
            return format!("{}", res.text().unwrap());
        } else {
            return format!("404");
        }
    } else {
        let ten_millis = Duration::from_millis(500);
        thread::sleep(ten_millis);
        return evaluate_python(submission);
    }
}

fn main() {
    let min_worker = 4u32;
    let max_worker = 16u32;
    let init = thread::spawn(move || {
        //let parent_lxc = worker::Worker::from_image("python-tester-image", "parent-tester");
        let parent_lxc = worker::Worker::load("parent-tester");
        parent_lxc.stop();
        for i in 0..min_worker {
            let new_worker = parent_lxc.copy(format!("child-tester-{:02}", i+1).as_str()); // TODO: needs a check for already existing and handling it
            // TODO: make snapshot
            let millis = Duration::from_millis(500);
            thread::sleep(millis);
            new_worker.start();
        }
    });

    init.join().unwrap();

    rocket::ignite().mount("/", routes![evaluate]).launch();
}
