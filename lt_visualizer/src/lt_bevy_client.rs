use std::{ops::Deref, sync::Arc};
use bevy::prelude::*;

use lt_client::client::LunaTechClient;

pub struct LTBevyClientPlugin {
    pub port: u16,
}

impl Plugin for LTBevyClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientPort(self.port));
        app.init_resource::<LTBevyClient>();
    }
}

#[derive(Resource)]
pub struct ClientPort(pub u16);
impl Deref for ClientPort {
    type Target = u16;

    fn deref(&self) -> &u16 {
        &self.0
    }
}

#[derive(Resource)]
pub struct LTBevyClient(Arc<LunaTechClient>);
impl Deref for LTBevyClient {
    type Target = Arc<LunaTechClient>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromWorld for LTBevyClient {
    fn from_world(world: &mut World) -> Self {
        let port = **world.resource::<ClientPort>();
        LTBevyClient(LunaTechClient::new(port).into())
    }
}
