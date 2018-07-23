extern crate gateway_lib;
extern crate stq_logging;

fn main() {
    let config = gateway_lib::config::Config::new().expect("Can't load gateway configs. Please check your /config folder.");

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    gateway_lib::start(config);
}
