use bevy::{color::palettes::tailwind::CYAN_100, prelude::*};

const RADIUS: f32 = 20.0;
const CONTROL_POINT_LIMIT: usize = 4;

#[derive(Component)]
struct StartingPoint;

#[derive(Component)]
struct EndPoint;

#[derive(Component)]
struct ControlPoint;

#[derive(Component)]
struct Point;

#[derive(Component)]
struct TypeOfCurve;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn)
        .add_systems(Update, (move_object, render_curve, add_or_remove_control_points))
        .insert_resource(ClearColor(Color::linear_rgb(0.0, 0.0, 0.0)))
        .run();
}

fn quadratic(a: Vec3, b: Vec3, c: Vec3, alpha: f32) -> Vec3 {
    let p0 = a.lerp(b, alpha);
    let p1 = b.lerp(c, alpha);
    p0.lerp(p1, alpha)
}

fn cubic(a: Vec3, b: Vec3, c: Vec3, d: Vec3, alpha: f32) -> Vec3 {
    let p0 = quadratic(a, b, c, alpha);
    let p1 = quadratic(b, c, d, alpha);
    p0.lerp(p1, alpha)
}

fn quartic(a: Vec3, b: Vec3, c: Vec3, d: Vec3, e: Vec3, alpha: f32) -> Vec3 {
    let p0 = cubic(a, b, c, d, alpha);
    let p1 = cubic(b, c, d, e, alpha);
    p0.lerp(p1, alpha)
}

fn quintic(a: Vec3, b: Vec3, c: Vec3, d: Vec3, e: Vec3, f: Vec3, alpha: f32) -> Vec3 {
    let p0 = quartic(a, b, c, d, e, alpha);
    let p1 = quartic(b, c, d, e, f, alpha);
    p0.lerp(p1, alpha)
}

fn render_curve(
    mut gizmos: Gizmos,
    a: Single<&mut Transform, (With<StartingPoint>, Without<ControlPoint>, Without<EndPoint>)>,
    b: Query<&mut Transform, (With<ControlPoint>, Without<StartingPoint>, Without<EndPoint>)>,
    c: Single<&mut Transform, (With<EndPoint>, Without<ControlPoint>, Without<StartingPoint>)>,
) {
    let n_points: usize = 500;
    let mut curve_points: Vec<Vec3> = Vec::with_capacity(n_points);
    let control_points: Vec<Vec3> = b.iter().map(|f| f.translation).collect();

    let l = control_points.len();
    let f = |alpha| {
        match l {
            1 => quadratic(a.translation, control_points[0], c.translation, alpha),
            2 => cubic(a.translation, control_points[0], control_points[1], c.translation, alpha),
            3 => quartic(a.translation, control_points[0], control_points[1], control_points[2], c.translation, alpha),
            4 => quintic(a.translation, control_points[0], control_points[1], control_points[2], control_points[3], c.translation, alpha),
            _ => panic!() // shouldnt ever happen
        }
    };

    for i in 0..n_points {
        let alpha = i as f32/n_points as f32;
        curve_points.push(f(alpha));
    }

    for point in curve_points.into_iter() {
        gizmos.circle(point, 1.0, CYAN_100);
    }
}

fn add_or_remove_control_points(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera>>,
    window: Single<&Window>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut n_controls: Local<usize>,

    control_points: Query<(&mut Transform, Entity), With<ControlPoint>>,
    mut type_of_curve: Single<&mut Text, With<TypeOfCurve>>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    if *n_controls == 0 {
        *n_controls = 1;
    }
    
    let last = *n_controls;
    if keyboard_input.just_pressed(KeyCode::ArrowUp) && *n_controls != CONTROL_POINT_LIMIT {
        let position = camera.0.viewport_to_world(camera.1, cursor_pos).unwrap();
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(RADIUS))),
            ControlPoint,
            Point,
            MeshMaterial2d(materials.add(Color::linear_rgb(255.0, 255.0, 255.0))),
            Transform::from_translation(position.origin),
        ));
        *n_controls += 1;
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        let wrt_world = camera.0.viewport_to_world_2d(camera.1, cursor_pos).unwrap();
        for (transform, entity) in control_points.iter() {
            if (transform.translation.x - wrt_world.x).powi(2) + (transform.translation.y - wrt_world.y).powi(2) < RADIUS * RADIUS && *n_controls > 1 {
                commands.entity(entity).despawn();
                *n_controls -= 1;
                break;
            }
        }
    }

    if n_controls.abs_diff(last) != 0 {
        match *n_controls {
            1 => type_of_curve.0 = "Quadratic".to_string(),
            2 => type_of_curve.0 = "Cubic".to_string(),
            3 => type_of_curve.0 = "Quartic".to_string(),
            4 => type_of_curve.0 = "Quintic".to_string(),
            _ => type_of_curve.0 = "Bigger than Quintic".to_string(),
        }
    }
}

fn move_object(
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut transforms: Query<&mut Transform, With<Point>>,
    camera: Single<(&GlobalTransform, &Camera), With<Camera>>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }
    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(wrt_world) = camera.1.viewport_to_world_2d(camera.0, cursor_pos).ok() {
            for mut transform in transforms.iter_mut() {
                if (transform.translation.x - wrt_world.x).powi(2) + (transform.translation.y - wrt_world.y).powi(2) < RADIUS * RADIUS
                {
                    transform.translation =
                        Vec3::new(wrt_world.x, wrt_world.y, transform.translation.z);
                        
                    let viewport_size = window.size() * 0.5;
                    let v_x = viewport_size.x;
                    let v_y = viewport_size.y;
                    
                    transform.translation = transform.translation.clamp(
                        Vec3::new(-v_x + RADIUS, -v_y + RADIUS, transform.translation.z),
                        Vec3::new(v_x - RADIUS, v_y - RADIUS, transform.translation.z),
                    );
                }
            }
        }
    }
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d,));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(RADIUS))),
        StartingPoint,
        Point,
        MeshMaterial2d(materials.add(Color::linear_rgb(255.0, 255.0, 255.0))),
        Transform::from_xyz(-200.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(RADIUS))),
        EndPoint,
        Point,
        MeshMaterial2d(materials.add(Color::linear_rgb(255.0, 255.0, 255.0))),
        Transform::from_xyz(200.0, 50.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(RADIUS))),
        ControlPoint,
        Point,
        MeshMaterial2d(materials.add(Color::linear_rgb(255.0, 255.0, 255.0))),
        Transform::from_xyz(0.0, 100.0, 0.0),
    ));

    commands.spawn((
        Text::new(
            "Quadratic",
        ),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::linear_rgb(0.0, 255.0, 0.0)),
        TypeOfCurve,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            ..default()
        },
    ));

    commands.spawn((
        Text::new(
            "UP/DOWN arrow to add/remove control points",
        ),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::linear_rgb(0.0, 255.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            ..default()
        },
    ));
}
