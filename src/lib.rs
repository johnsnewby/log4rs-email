//! Email appender for  log4rs.

extern crate gethostname;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
#[cfg(feature = "file")]
extern crate serde;
#[cfg(feature = "file")]
#[macro_use]
extern crate serde_derive;

pub mod log4rs_email;
