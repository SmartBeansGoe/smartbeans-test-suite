use serde::Deserialize;
use serde::Serialize;
use std::cmp::PartialEq;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SmartResult {
    Undefined,
    ParsingError,
    UnknownError,
    EvaluationError,
    ContainerError,
    Success,
    Failed,
    TimedOut,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SmartSubmission {
    pub source_code: String,
    pub tests: String,
    pub timeout: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SmartResponse {
    pub score: u32,
    pub result_type: SmartResult,
    pub runs: u32,
    pub errors: u32,
    pub failures: u32,
    pub feedback: String,
}

impl fmt::Display for SmartResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"score\": {}, \"result_type\": {:?}, \"runs\": {}, \"errors\": {}, \"failures\": {}, \"feedback\": {}}}",
            self.score, self.result_type, self.runs, self.errors, self.failures, self.feedback
        )
    }
}

impl SmartResponse {
    // Updates the result_type.
    pub fn update(mut self) -> Self {
        if self.result_type != SmartResult::EvaluationError
            && self.result_type != SmartResult::TimedOut
            && self.result_type != SmartResult::ParsingError
            && self.result_type != SmartResult::UnknownError
        {
            if self.errors == 0 && self.failures == 0 {
                self.result_type = SmartResult::Success;
                self.score = 1;
            } else {
                self.result_type = SmartResult::Failed
            }
        }
        self
    }

    // Generates the SmartResponse from json output of the test run.
    // Expected is a JSON which contains all entries of SmartResponse.
    // The result_type could be set as unknown in the test.
    pub fn from_output(output: std::process::Output) -> SmartResponse {
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
                    result_type = SmartResult::UnknownError;
                    feedback = "UnknownError".to_string();
                }
            };
            SmartResponse {
                score: 0,
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
