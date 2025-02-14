use std::sync::Mutex;
use bevy::{
    color::palettes::css::*,
    math::Isometry2d,
    prelude::*,
};
use lazy_static::lazy_static;

use lt_client::client::LunaTechClient;

const SPIN_SPEED: f32 = 100.;
lazy_static! {
    static ref CLIENT: Mutex<LunaTechClient> = Mutex::new(LunaTechClient::new(3000));
    static ref SPINMULT: Mutex<f32> = Mutex::new(0.);
}

fn main() {
    CLIENT.lock().unwrap().start_client();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_controls)
        .add_systems(
            Update,
            spin,
        )
        .add_systems(
            PostUpdate,
            (
                render_shapes,
            )
                .chain(),
        )
        .run();
}


fn update_controls() {
    let client = CLIENT.lock().unwrap();
    let audio_features = client.get_audio_features().unwrap();
    let mut sm = SPINMULT.lock().unwrap();
    let _ = sm.set(Box::new(audio_features.high_range_rms));
}


#[derive(Component)]
struct Spin;

fn spin(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        let spin_mult = *SPINMULT.lock().unwrap();
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() / 5. * SPIN_SPEED * spin_mult);
    }
}

#[derive(Component)]
enum Shape {
    Rectangle(Rectangle),
    Circle(Circle),
    Triangle(Triangle2d),
    Line(Segment2d),
    Capsule(Capsule2d),
    Polygon(RegularPolygon),
}

fn render_shapes(mut gizmos: Gizmos, query: Query<(&Shape, &Transform)>) {
    let color = GRAY;
    for (shape, transform) in query.iter() {
        let translation = transform.translation.xy();
        let rotation = transform.rotation.to_euler(EulerRot::YXZ).2;
        let isometry = Isometry2d::new(translation, Rot2::radians(rotation));
        match shape {
            Shape::Rectangle(r) => {
                gizmos.primitive_2d(r, isometry, color);
            }
            Shape::Circle(c) => {
                gizmos.primitive_2d(c, isometry, color);
            }
            Shape::Triangle(t) => {
                gizmos.primitive_2d(t, isometry, color);
            }
            Shape::Line(l) => {
                gizmos.primitive_2d(l, isometry, color);
            }
            Shape::Capsule(c) => {
                gizmos.primitive_2d(c, isometry, color);
            }
            Shape::Polygon(p) => {
                gizmos.primitive_2d(p, isometry, color);
            }
        }
    }
}

const OFFSET_X: f32 = 125.;
const OFFSET_Y: f32 = 75.;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Transform::from_xyz(-OFFSET_X, OFFSET_Y, 0.),
        Shape::Circle(Circle::new(45.)),
    ));

    commands.spawn((
        Transform::from_xyz(0., OFFSET_Y, 0.),
        Shape::Rectangle(Rectangle::new(80., 80.)),
        Spin,
    ));

    commands.spawn((
        Transform::from_xyz(OFFSET_X, OFFSET_Y, 0.),
        Shape::Triangle(Triangle2d::new(
            Vec2::new(-40., -40.),
            Vec2::new(-20., 40.),
            Vec2::new(40., 50.),
        )),
        Spin,
    ));

    commands.spawn((
        Transform::from_xyz(-OFFSET_X, -OFFSET_Y, 0.),
        Shape::Line(Segment2d::new(Dir2::from_xy(1., 0.3).unwrap(), 90.)),
        Spin,
    ));

    commands.spawn((
        Transform::from_xyz(0., -OFFSET_Y, 0.),
        Shape::Capsule(Capsule2d::new(25., 50.)),
        Spin,
    ));

    commands.spawn((
        Transform::from_xyz(OFFSET_X, -OFFSET_Y, 0.),
        Shape::Polygon(RegularPolygon::new(50., 6)),
        Spin,
    ));
}
