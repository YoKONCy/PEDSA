#![allow(dead_code)]
pub trait AsyncTaskInterface {
    fn schedule_maintenance(&self, context: &str);
}
