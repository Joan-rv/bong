use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
    prelude::*,
};
use std::f32::consts::{FRAC_PI_2, PI};

const WALL_WIDTH: f32 = 10.;
const WALL_OFFSET: f32 = 220.;
const WALL_LENGTH: f32 = WALL_OFFSET * 2. + WALL_WIDTH;

const PADDLE_SPEED: f32 = 200.;
const PADDLE_OFFSET: f32 = 200.;
const PADDLE_SIZE: Vec2 = Vec2::new(10., 50.);

const BALL_SIZE: f32 = 20.;
const BALL_SPEED: f32 = 200.;

const TEXT_PADDING: Val = Val::Px(20.);
const TEXT_SIZE: f32 = 36.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (apply_velocity, move_paddles),
                detect_collisions,
                update_score,
            )
                .chain(),
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

#[derive(Component)]
struct Wall;

#[derive(Resource)]
struct Score(u32, u32);

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
        Velocity(Vec2::new(BALL_SPEED, 0.)),
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

    let mut create_wall = |x, y, width, height| {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(materials.add(Color::WHITE)),
            Transform::from_xyz(x, y, 0.).with_scale(Vec2::new(width, height).extend(1.)),
            Wall,
            Collider,
        ));
    };
    create_wall(WALL_OFFSET, 0., WALL_WIDTH, WALL_LENGTH);
    create_wall(-WALL_OFFSET, 0., WALL_WIDTH, WALL_LENGTH);
    create_wall(0., WALL_OFFSET, WALL_LENGTH, WALL_WIDTH);
    create_wall(0., -WALL_OFFSET, WALL_LENGTH, WALL_WIDTH);

    commands.spawn((
        Text::new("0 - 0"),
        TextFont {
            font_size: TEXT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: TEXT_PADDING,
            left: TEXT_PADDING,
            ..default()
        },
    ));

    commands.insert_resource(Score(0, 0));
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
    mut scores: ResMut<Score>,
    ball: Single<(&mut Transform, &mut Velocity), (With<Ball>, Without<Collider>)>,
    colliders: Query<(&Transform, Option<&Paddle>), With<Collider>>,
) {
    let (mut ball_transform, mut ball_velocity) = ball.into_inner();
    let ball_radius = ball_transform.scale.x / 2.;

    let bounding_circle = BoundingCircle::new(ball_transform.translation.xy(), ball_radius);
    for (transform, maybe_paddle) in colliders {
        let bounding_box = Aabb2d::new(transform.translation.xy(), transform.scale.xy() / 2.);
        if bounding_circle.intersects(&bounding_box) {
            enum Side {
                Left,
                Right,
            }
            let side = if ball_transform.translation.x < transform.translation.x {
                Side::Left
            } else {
                Side::Right
            };

            if maybe_paddle.is_some() {
                let angle = (PI * rand::random::<f32>() - FRAC_PI_2) / 2.;
                **ball_velocity = match side {
                    Side::Right => Vec2::from_angle(angle) * BALL_SPEED,
                    Side::Left => Vec2::from_angle(angle + PI) * BALL_SPEED,
                };
            } else {
                ball_velocity.y *= -1.;
                if ball_transform.translation.x - ball_radius < -WALL_OFFSET
                    || ball_transform.translation.x + ball_radius > WALL_OFFSET
                {
                    ball_transform.translation = Vec3::ZERO;
                    match side {
                        Side::Left => scores.0 += 1,
                        Side::Right => scores.1 += 1,
                    };
                }
            }

            break;
        }
    }
}

fn update_score(scores: Res<Score>, mut text: Single<&mut Text>) {
    text.0 = format!("{} - {}", scores.0, scores.1);
}
