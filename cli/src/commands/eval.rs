use std::error::Error;

pub fn execute(code: &str) -> Result<(), Box<dyn Error>> {
    mdeno_runtime::eval_code(code)
}
