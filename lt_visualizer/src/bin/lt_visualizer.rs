use bevy::prelude::*;
use lt_visualizer::lt_bevy_client::{LTBevyClient, LTBevyClientPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LTBevyClientPlugin { port: 3000 })
        .add_systems(Update, test_system) 
        .run();
}

fn test_system(client: Res<LTBevyClient>) {
    let _ = client.audio_features.broad_range_peak_rms.get();
}

