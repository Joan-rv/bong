use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
    prelude::*,
};

const PADDLE_SPEED: f32 = 200.;
const PADDLE_OFFSET: f32 = 200.;
const PADDLE_SIZE: Vec2 = Vec2::new(10., 50.);

const BALL_SIZE: f32 = 20.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            ((apply_velocity, move_paddles), detect_collisions).chain(),
        )
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle {
    up: KeyCode,
    down: KeyCode,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_xyz(0., 0., 0.).with_scale(Vec2::splat(BALL_SIZE).extend(1.)),
        Ball,
        Velocity(Vec2::new(100., 100.)),
    ));
    let mut create_paddle = |x, up, down| {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(materials.add(Color::WHITE)),
            Transform::from_xyz(x, 0., 0.).with_scale(PADDLE_SIZE.extend(1.)),
            Paddle { up, down },
            Collider,
        ));
    };

    create_paddle(PADDLE_OFFSET, KeyCode::ArrowUp, KeyCode::ArrowDown);
    create_paddle(-PADDLE_OFFSET, KeyCode::KeyW, KeyCode::KeyS);
}

fn apply_velocity(query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn move_paddles(
    query: Query<(&mut Transform, &Paddle)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, paddle) in query {
        if keyboard.pressed(paddle.up) {
            transform.translation.y += PADDLE_SPEED * time.delta_secs();
        }
        if keyboard.pressed(paddle.down) {
            transform.translation.y -= PADDLE_SPEED * time.delta_secs();
        }
    }
}

fn detect_collisions(
    mut ball: Single<(&Transform, &mut Velocity), With<Ball>>,
    colliders: Query<&Transform, With<Collider>>,
) {
    let bounding_circle = BoundingCircle::new(ball.0.translation.xy(), ball.0.scale.x / 2.);
    for transform in colliders {
        let bounding_box = Aabb2d::new(transform.translation.xy(), transform.scale.xy() / 2.);
        if bounding_circle.intersects(&bounding_box) {
            **ball.1 *= -1.;
            break;
        }
    }
}
