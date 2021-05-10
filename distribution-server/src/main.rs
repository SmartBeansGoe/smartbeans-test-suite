#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

use lxd::Location;
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use rocket_contrib::json::Json;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
struct Submission {
    source_code: String,
    tests: String,
    timeout: String,
}

lazy_static! {
    static ref WORKER_IDLING: Mutex<Vec<worker::Worker>> = Mutex::new(vec![]);
    static ref WORKER_RESET: Mutex<Vec<worker::Worker>> = Mutex::new(vec![]);
    static ref RESET_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref WORKER_RESET_THREAD: thread::JoinHandle<()> = thread::spawn(|| {
        while !RESET_FLAG.load(Ordering::Acquire) {
            thread::park();
            while WORKER_RESET.lock().unwrap().len() > 0 {
                let container: worker::Worker = WORKER_RESET.lock().unwrap().pop().unwrap();
                container.restore("snap0");
                WORKER_IDLING.lock().unwrap().push(container);
            }
        }
    });


}

mod worker;

#[post("/evaluate/<lang>", format = "application/json", data = "<submission>")]
fn evaluate(lang: String, submission: Json<Submission>) -> String {
    match lang.to_lowercase().as_ref() {
        "python" => evaluate_python(submission),
        _ => format!("Could not find test suite for language: {}", lang),
    }
}

fn evaluate_python(submission: Json<Submission>) -> String {
    let container: Option<worker::Worker> = WORKER_IDLING.lock().unwrap().pop();
    if let Some(container) = container {
        let client = reqwest::blocking::Client::new();
        let ip = container.ipv4().to_string();
        let res = client
            .post(format!("http://{}:8000/evaluate", ip))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(format!("{:?}", submission).to_string())
            .send();

        let res = res.unwrap();

        WORKER_RESET.lock().unwrap().push(container);
        WORKER_RESET_THREAD.thread().unpark();

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
    let num_worker = 16u32;

    let init = thread::spawn(move || {
        for i in 0..num_worker {
            let new_worker = worker::Worker::new(
                Location::Local,
                format!("child-tester-{:02}", i + 1).as_str(),
                "python-tester",
            );
            //new_worker.profile("tester");
            new_worker.snapshot("snap0");
            WORKER_IDLING.lock().unwrap().push(new_worker);
        }
    });

    init.join().unwrap();

    rocket::ignite().mount("/", routes![evaluate]).launch();
    //WORKER_RESET_THREAD.join().unwrap();
}
