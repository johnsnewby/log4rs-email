//! Email appender for  log4rs.

extern crate gethostname;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
#[cfg(feature = "file")]
extern crate serde;

pub mod log4rs_email;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Foo {
    x: String,
}
