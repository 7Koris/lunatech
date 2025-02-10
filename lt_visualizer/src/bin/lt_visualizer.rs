use lt_client::client::LunaTechClient;
use bevy::prelude::*;

fn main() {
    let client = LunaTechClient::new(3000);
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
    ))
    .add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}