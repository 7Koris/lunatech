use std::{thread::{self, sleep}, time::Duration};

use lt_client::client::LunaTechClient;
use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum MyAppState {
    Disconnected,
    AnimatingBootScreen, // Running boot animation
    Running,
    Faulted,
}

// TODO: Launch server if not already running or launch on arg
fn main() {
    let client = LunaTechClient::new(3000);
    client.start_client();
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
    ))
    //.add_event
    .add_systems(Startup, setup);

    thread::sleep(Duration::from_secs(5));

    app.run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}