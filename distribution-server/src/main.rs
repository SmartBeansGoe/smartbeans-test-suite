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
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use rocket_contrib::json::Json;
use smart::{SmartResponse, SmartResult, SmartSubmission};

lazy_static! {
    static ref WORKER_IDLING: Mutex<VecDeque<worker::Worker>> = Mutex::new(VecDeque::new());
    static ref WORKER_RESET: Mutex<Vec<worker::Worker>> = Mutex::new(vec![]);
    static ref RESET_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref WORKER_RESET_THREAD: thread::JoinHandle<()> = thread::spawn(|| {
        while !RESET_FLAG.load(Ordering::Acquire) {
            thread::park();
            while WORKER_RESET.lock().unwrap().len() > 0 {
                let container: worker::Worker = WORKER_RESET.lock().unwrap().pop().unwrap();
                container.restore("snap0");
                container.profile("tester");
                WORKER_IDLING.lock().unwrap().push_back(container);
            }
        }
    });
}

mod worker;

#[post("/evaluate/<lang>", format = "application/json", data = "<submission>")]
fn evaluate(lang: String, submission: Json<SmartSubmission>) -> String {
    match lang.to_lowercase().as_ref() {
        "python" => evaluate_python(submission.into_inner()),
        _ => format!("Could not find test suite for language: {}", lang),
    }
}

fn evaluate_python(submission: SmartSubmission) -> String {
    let container: Option<worker::Worker> = WORKER_IDLING.lock().unwrap().pop_front();
    if let Some(container) = container {
        let container_name = container.container.name().to_string();
        let client = reqwest::blocking::Client::new();
        let ip = container.ipv4().to_string();
        let res = client
            .post(format!("http://{}:8000/evaluate", ip))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&submission)
            .send();

        let res = res.unwrap();

        WORKER_RESET.lock().unwrap().push(container);
        WORKER_RESET_THREAD.thread().unpark();

        println!("{:?}", res);

        if res.status().is_success() {
            return format!("{}", res.text().unwrap());
        } else {
            return serde_json::to_string(&SmartResponse {
                result_type: SmartResult::ContainerError,
                errors: 0,
                failures: 0,
                runs: 0,
                score: 0,
                feedback: format!("ContainerError: The container \"{}\" which was contacted, was not reachable under the IP.", container_name)
            }).unwrap();
        }
    } else {
        let waiting_time = Duration::from_millis(500);
        thread::sleep(waiting_time);
        return evaluate_python(submission);
    }
}

fn main() {
    // Load environment variables from .env
    dotenv::dotenv().ok();
    
    let num_worker = std::env::var("NUM_WORKER").unwrap().parse::<u32>().unwrap();

    thread::spawn(move || {
        for i in 0..num_worker {
            let new_worker = worker::Worker::new(
                Location::Local,
                format!("child-tester-{:02}", i + 1).as_str(),
                "python-tester",
            );
            //new_worker.profile("tester");
            new_worker.snapshot("snap0");
            WORKER_IDLING.lock().unwrap().push_back(new_worker);
        }
    });

    rocket::ignite().mount("/", routes![evaluate]).launch();
    //WORKER_RESET_THREAD.join().unwrap();
}
