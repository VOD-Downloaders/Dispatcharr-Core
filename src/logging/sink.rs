use std::sync::Mutex;

/////////////////////////////////////////////////////
// LogLevel
/////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel
{
    Trace,
    Info,
    Warn,
    Error,
}

/////////////////////////////////////////////////////
// Sink types
/////////////////////////////////////////////////////
pub trait Sink: Send
{
    fn log(&mut self, log_level: LogLevel, message: &str);
}

/////////////////////////////////////////////////////
// Sinks
/////////////////////////////////////////////////////
static SINKS: Mutex<Vec<Box<dyn Sink>>> = Mutex::new(Vec::new());

pub fn add_sink(sink: Box<dyn Sink>)
{
    if let Ok(mut sinks) = SINKS.lock()
    {
        sinks.push(sink);
    }
}

pub fn clear_sinks()
{
    if let Ok(mut sinks) = SINKS.lock()
    {
        sinks.clear();
    }
}

pub fn log_to_all_sinks(log_level: LogLevel, message: &str)
{
    if let Ok(mut sinks) = SINKS.lock()
    {
        for sink in sinks.iter_mut()
        {
            sink.log(log_level.clone(), message);
        }
    }
}
