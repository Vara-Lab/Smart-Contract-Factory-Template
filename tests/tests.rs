use gstd::{prelude::*, ActorId};
use gtest::{Log, Program, System};
use io::*;

#[test]
fn test() {
    let system = System::new();

    system.init_logger();

    let state_binary = get_state_binary();
    let program = Program::current(&system);

    let mut result = program.send_bytes(2, []);

    assert!(!result.main_failed());

    result = program.send(2, PingPong::Pong);

    assert!(!result.main_failed());
}
