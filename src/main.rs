pub mod draw;
// From demo.

use std::time::Duration;

use bevy::{
    app::{App, Startup, Update},
    color::{
        palettes::css::{BLACK, WHITE},
        *,
    },
    core_pipeline::{fxaa::Fxaa, smaa::Smaa},
    ecs::system::Commands,
    gizmos::gizmos::Gizmos,
    input::{ButtonState, mouse::MouseButtonInput, touch::TouchPhase},
    math::{cubic_splines::*, vec2},
    prelude::*,
    winit::WinitSettings,
};
use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy_prototype_lyon::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 24.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::Srgba(Srgba {
                        red: 0.,
                        green: 1.,
                        blue: 0.,
                        alpha: 1.,
                    }),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
            ShapePlugin
        ))
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_keypress,
                handle_mouse_move,
                handle_touch_state,
                handle_mouse_press,
                //draw_edit_move,
                update_spline_mode_text,
                update_cycling_mode_text,
                curve_fill,
                draw_curve,
                //draw_control_points,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Initialize the modes with their defaults:
    let spline_mode = SplineMode::default();
    commands.insert_resource(spline_mode);
    let cycling_mode = CyclingMode::default();
    commands.insert_resource(cycling_mode);
    commands.insert_resource(TouchMove::default());

    // Starting data for [`ControlPoints`]:
    let default_points = vec![
        vec2(-500., -200.),
        vec2(-250., 250.),
        vec2(250., 250.),
        vec2(500., -200.),
    ];

    let default_tangents = vec![
        vec2(0., 200.),
        vec2(200., 0.),
        vec2(0., -200.),
        vec2(-200., 0.),
    ];

    let default_control_data = CurrentCurve {
        points_and_tangents: default_points.into_iter().zip(default_tangents).collect(),
    };

    //let curve = form_curve(&default_control_data, spline_mode, cycling_mode);
    //if let Some(curve) = curve {
    //    commands.insert_resource(curve);
    //}
    commands.insert_resource(default_control_data);

    // Mouse tracking information:
    commands.insert_resource(MousePosition::default());
    commands.insert_resource(MouseEditMove::default());

    commands.spawn(Camera2d).insert(Fxaa::default());

    // The instructions and modes are rendered on the left-hand side in a column.
    let instructions_text = "Draw on the screen.\n\
        R: Remove the last control point\n";
    let spline_mode_text = format!("Spline: {spline_mode}");
    let cycling_mode_text = format!("{cycling_mode}");
    let style = TextFont::default();

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((Text::new(instructions_text), style.clone()));
            parent.spawn((SplineModeText, Text(spline_mode_text), style.clone()));
            parent.spawn((CyclingModeText, Text(cycling_mode_text), style.clone()));
        });
}

// -----------------------------------
// Curve-related Resources and Systems
// -----------------------------------

/// The current spline mode, which determines the spline method used in conjunction with the
/// control points.
#[derive(Clone, Copy, Resource, Default)]
enum SplineMode {
    Hermite,
    #[default]
    Cardinal,
    B,
}

impl std::fmt::Display for SplineMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplineMode::Hermite => f.write_str("Hermite"),
            SplineMode::Cardinal => f.write_str("Cardinal"),
            SplineMode::B => f.write_str("B"),
        }
    }
}

/// The current cycling mode, which determines whether the control points should be interpolated
/// cyclically (to make a loop).
#[derive(Clone, Copy, Resource, Default)]
enum CyclingMode {
    #[default]
    NotCyclic,
    Cyclic,
}

impl std::fmt::Display for CyclingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CyclingMode::NotCyclic => f.write_str("Not Cyclic"),
            CyclingMode::Cyclic => f.write_str("Cyclic"),
        }
    }
}

/// Finished curves.
#[derive(Clone, Component)]
struct Curve(CubicCurve<Vec2>);

#[derive(Clone, Component)]
struct ProcessedCurve {
    interp: Vec<Vec2>,
}

#[derive(Clone, Component)]
struct CurrentCurveMarker;

#[derive(Clone, Component)]
struct LyonMarker;
/// The control points used to generate a curve. The tangent components are only used in the case of
/// Hermite interpolation.
#[derive(Clone, Resource)]
struct CurrentCurve {
    points_and_tangents: Vec<(Vec2, Vec2)>,
}

/// This system is responsible for updating the [`Curve`] when the [control points] or active modes
/// change.
///
/// [control points]: ControlPoints
//fn update_curve(
//    control_points: Res<CurrentCurve>,
//    spline_mode: Res<SplineMode>,
//    cycling_mode: Res<CyclingMode>,
//    mut curve: Query<&Curve,With<Curve>>,
//) {
//    if !control_points.is_changed() && !spline_mode.is_changed() && !cycling_mode.is_changed() {
//        return;
//    }
//
//    let next_curve = form_curve(&control_points, *spline_mode, *cycling_mode);
//}

/// This system uses gizmos to draw the current [`Curve`] by breaking it up into a large number
/// of line segments.
fn draw_curve(
    curves: Query<
        (&ProcessedCurve, Entity),
        (With<Curve>, With<ProcessedCurve>, Without<LyonMarker>),
    >,
    mut gizmos: Gizmos,
    current: Res<CurrentCurve>,
    par_cmd: ParallelCommands,
    mut commands: Commands,
) {
    // Scale resolution with curve length so it doesn't degrade as the length increases.

    // Gizmos is not parallel.
    //for curve in curves {
    //let resolution = 100 * curve..segments().len();
    //gizmos.linestrip(
    //    curve.0.iter_positions(resolution).map(|pt| pt.extend(0.0)),
    //    Color::srgb(1.0, 1.0, 1.0),
    //);
    //gizmos.curve_2d(
    //    &curve.0,
    //    (0..resolution).map(|n| n as f32 / 100.0),
    //    Color::srgb(1.0, 1.0, 1.0),
    //);
    //gizmos.linestrip_2d(curve.interp.iter().map(|x| *x), Color::srgb(1.0, 1.0, 1.0));
    //}
    curves.iter().for_each(|curve| {
        let mut path = ShapePath::new().move_to(curve.0.interp[0]);
        for i in curve.0.interp[1..].iter() {
            path = path.line_to(i.clone());
            //trace!("point {}",i);
        }

        commands.entity(curve.1).insert((
            ShapeBuilder::with(&path).stroke((WHITE, 3.0)).build(),
            Transform::from_xyz(0., 0., 0.),
            LyonMarker,
        ));
    });
    // Form current curve. To be optimized.
    let Some(curve) = form_curve(&current, SplineMode::Cardinal, CyclingMode::NotCyclic) else {
        return;
    };
    let resolution = 3 * curve.0.segments().len();
    gizmos.curve_2d(
        &curve.0,
        (0..resolution).map(|n| n as f32 / 3. as f32),
        Color::srgb(1.0, 1.0, 1.0),
    );
}

/// This system uses gizmos to draw the current [control points] as circles, displaying their
/// tangent vectors as arrows in the case of a Hermite spline.
///
/// [control points]: ControlPoints
fn draw_control_points(control_points: Res<CurrentCurve>, spline_mode: Res<SplineMode>) {
    for &(point, tangent) in &control_points.points_and_tangents {
        //gizmos.circle_2d(point, 10.0, Color::srgb(0.0, 1.0, 0.0));

        if matches!(*spline_mode, SplineMode::Hermite) {
            //gizmos.arrow_2d(point, point + tangent, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

/// Helper function for generating a [`Curve`] from [control points] and selected modes.
///
/// [control points]: ControlPoints
fn form_curve(
    control_points: &CurrentCurve,
    spline_mode: SplineMode,
    cycling_mode: CyclingMode,
) -> Option<Curve> {
    let (points, tangents): (Vec<_>, Vec<_>) =
        control_points.points_and_tangents.iter().copied().unzip();

    match spline_mode {
        SplineMode::Hermite => {
            let spline = CubicHermite::new(points, tangents);
            match cycling_mode {
                CyclingMode::NotCyclic => spline.to_curve().ok(),
                CyclingMode::Cyclic => spline.to_curve_cyclic().ok(),
            }
            .map(|x| Curve(x))
        }
        SplineMode::Cardinal => {
            let spline = CubicCardinalSpline::new_catmull_rom(points);
            match cycling_mode {
                CyclingMode::NotCyclic => spline.to_curve().ok(),
                CyclingMode::Cyclic => spline.to_curve_cyclic().ok(),
            }
            .map(|x| Curve(x))
        }
        SplineMode::B => {
            let spline = CubicBSpline::new(points);
            match cycling_mode {
                CyclingMode::NotCyclic => spline.to_curve().ok(),
                CyclingMode::Cyclic => spline.to_curve_cyclic().ok(),
            }
            .map(|x| Curve(x))
        }
    }
}

fn curve_with_lyon(control_points: &CurrentCurve, mut commands: Commands) {}

// --------------------
// Text-related Components and Systems
// --------------------

/// Marker component for the text node that displays the current [`SplineMode`].
#[derive(Component)]
struct SplineModeText;

/// Marker component for the text node that displays the current [`CyclingMode`].
#[derive(Component)]
struct CyclingModeText;

fn update_spline_mode_text(
    spline_mode: Res<SplineMode>,
    mut spline_mode_text: Query<&mut Text, With<SplineModeText>>,
) {
    if !spline_mode.is_changed() {
        return;
    }

    let new_text = format!("Spline: {}", *spline_mode);

    for mut spline_mode_text in spline_mode_text.iter_mut() {
        (**spline_mode_text).clone_from(&new_text);
    }
}

fn update_cycling_mode_text(
    cycling_mode: Res<CyclingMode>,
    mut cycling_mode_text: Query<&mut Text, With<CyclingModeText>>,
) {
    if !cycling_mode.is_changed() {
        return;
    }

    let new_text = format!("{}", *cycling_mode);

    for mut cycling_mode_text in cycling_mode_text.iter_mut() {
        (**cycling_mode_text).clone_from(&new_text);
    }
}

// -----------------------------------
// Input-related Resources and Systems
// -----------------------------------

/// A small state machine which tracks a click-and-drag motion used to create new control points.
///
/// When the user is not doing a click-and-drag motion, the `start` field is `None`. When the user
/// presses the left mouse button, the location of that press is temporarily stored in the field.
#[derive(Clone, Default, Resource)]
struct MouseEditMove {
    start: Option<Vec2>,
}

#[derive(Clone, Default, Resource)]
struct TouchMove {
    which: Option<u64>,
}

/// The current mouse position, if known.
#[derive(Clone, Default, Resource)]
struct MousePosition(Option<Vec2>);

/// Update the current cursor position and track it in the [`MousePosition`] resource.
fn handle_mouse_move(
    mut cursor_events: EventReader<CursorMoved>,
    mut touch_events: EventReader<TouchInput>,
    mut mouse_position: ResMut<MousePosition>,

    mut control_points: ResMut<CurrentCurve>,
    touch_state: ResMut<TouchMove>,
    edit_move: Res<MouseEditMove>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    if let Some(cursor_event) = cursor_events.read().last() {
        // Only push points when mouse down.
        mouse_position.0 = Some(cursor_event.position);
        let Some(_) = edit_move.start else {
            return;
        };
        let (camera, camera_transform) = *camera;
        // Put current position as control point.
        let Ok(current) = camera.viewport_to_world_2d(camera_transform, cursor_event.position)
        else {
            return;
        };
        control_points
            .points_and_tangents
            .push((current, vec2(0., 0.)));
    }

    debug!("Reading movements...");
    for touch_event in touch_events.read() {
        debug!("touch {:?}", touch_event);
        if Some(touch_event.id) == touch_state.which {
            // Consider only the first touch (single touch)
            mouse_position.0 = Some(touch_event.position);
            let Some(_) = edit_move.start else {
                return;
            };
            let (camera, camera_transform) = *camera;
            let Ok(current) = camera.viewport_to_world_2d(camera_transform, touch_event.position)
            else {
                return;
            };
            control_points
                .points_and_tangents
                .push((current, vec2(0., 0.)));
        }
    }
}

/// This system handles updating the [`MouseEditMove`] resource, orchestrating the logical part
/// of the click-and-drag motion which actually creates new control points.
fn handle_mouse_press(
    mut button_events: EventReader<MouseButtonInput>,
    //mut touch_events: EventReader<TouchInput>, // Add this line
    mouse_position: Res<MousePosition>,
    mut edit_move: ResMut<MouseEditMove>,
    mut current_strip: ResMut<CurrentCurve>,
    mut commands: Commands,
    //mut touch_state: ResMut<TouchMove>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let Some(mouse_pos) = mouse_position.0 else {
        return;
    };

    // Handle click and drag behavior
    for button_event in button_events.read() {
        if button_event.button != MouseButton::Left {
            continue;
        }

        match button_event.state {
            ButtonState::Pressed => {
                if edit_move.start.is_some() {
                    // If the edit move already has a start, press event should do nothing.
                    continue;
                }
                // This press represents the start of the edit move.
                let (camera, camera_transform) = *camera;
                edit_move.start = Some(mouse_pos);
                let Ok(start_point) = camera.viewport_to_world_2d(camera_transform, mouse_pos)
                else {
                    continue;
                };
                current_strip.points_and_tangents.clear();
                current_strip
                    .points_and_tangents
                    .push((start_point, vec2(0., 0.)));
            }

            ButtonState::Released => {
                // Release is only meaningful if we started an edit move.
                let Some(start) = edit_move.start else {
                    continue;
                };

                let (camera, camera_transform) = *camera;

                // Convert the starting point and end point (current mouse pos) into world coords:
                let Ok(point) = camera.viewport_to_world_2d(camera_transform, start) else {
                    continue;
                };
                let Ok(end_point) = camera.viewport_to_world_2d(camera_transform, mouse_pos) else {
                    continue;
                };
                let tangent = end_point - point;

                // The start of the click-and-drag motion represents the point to add,
                // while the difference with the current position represents the tangent.
                //control_points.points_and_tangents.push((point, tangent));

                // Reset the edit move since we've consumed it.
                edit_move.start = None;

                let curve =
                    form_curve(&current_strip, SplineMode::Cardinal, CyclingMode::NotCyclic);
                if let Some(curve) = curve {
                    commands.spawn((Curve(curve.0),));
                }
                current_strip.points_and_tangents.clear();
            }
        }
    }
}

fn handle_touch_state(
    mut touch_events: EventReader<TouchInput>, // Add this line
    mut edit_move: ResMut<MouseEditMove>,
    mut current_strip: ResMut<CurrentCurve>,
    mut commands: Commands,
    mut touch_state: ResMut<TouchMove>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    //let Some(mouse_pos) = mouse_position.0 else {
    //    return;
    //};

    // Handle click and drag behavior

    for touch_event in touch_events.read() {
        // Consider only the first touch (single touch)
        debug!("Touch event {:?}", touch_event);
        match touch_event.phase {
            TouchPhase::Started => {
                if touch_state.which.is_some() {
                    continue;
                }
                let (camera, camera_transform) = *camera;
                edit_move.start = Some(touch_event.position);
                touch_state.which = Some(touch_event.id);
                let Ok(start_point) =
                    camera.viewport_to_world_2d(camera_transform, touch_event.position)
                else {
                    continue;
                };
                current_strip.points_and_tangents.clear();
                current_strip
                    .points_and_tangents
                    .push((start_point, vec2(0., 0.)));
            }
            TouchPhase::Ended => {
                let Some(start) = edit_move.start else {
                    continue;
                };

                let (camera, camera_transform) = *camera;

                let Ok(point) = camera.viewport_to_world_2d(camera_transform, start) else {
                    continue;
                };
                let Ok(end_point) =
                    camera.viewport_to_world_2d(camera_transform, touch_event.position)
                else {
                    continue;
                };
                let tangent = end_point - point;

                edit_move.start = None;
                touch_state.which = None;

                let curve =
                    form_curve(&current_strip, SplineMode::Cardinal, CyclingMode::NotCyclic);
                if let Some(curve) = curve {
                    commands.spawn((Curve(curve.0),));
                }
                current_strip.points_and_tangents.clear();
            }
            _ => {} // Do nothing for Moved or Cancelled phases here
        }
    }
}

/// This system handles drawing the "preview" control point based on the state of [`MouseEditMove`].
fn draw_edit_move(
    edit_move: Res<MouseEditMove>,
    mouse_position: Res<MousePosition>,
    mut gizmos: Gizmos,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    //let Some(start) = edit_move.start else {
    //    return;
    //};
    //let Some(mouse_pos) = mouse_position.0 else {
    //    return;
    //};

    //let (camera, camera_transform) = *camera;

    //// Resources store data in viewport coordinates, so we need to convert to world coordinates
    //// to display them:
    //let Ok(start) = camera.viewport_to_world_2d(camera_transform, start) else {
    //    return;
    //};
    //let Ok(end) = camera.viewport_to_world_2d(camera_transform, mouse_pos) else {
    //    return;
    //};

    //gizmos.circle_2d(start, 10.0, Color::srgb(0.0, 1.0, 0.7));
    //gizmos.circle_2d(start, 7.0, Color::srgb(0.0, 1.0, 0.7));
    //gizmos.arrow_2d(start, end, Color::srgb(1.0, 0.0, 0.7));
}

/// This system handles all keyboard commands.
fn handle_keypress(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut spline_mode: ResMut<SplineMode>,
    mut cycling_mode: ResMut<CyclingMode>,
    mut control_points: ResMut<CurrentCurve>,
) {
    // S => change spline mode
    if keyboard.just_pressed(KeyCode::KeyS) {
        *spline_mode = match *spline_mode {
            SplineMode::Hermite => SplineMode::Cardinal,
            SplineMode::Cardinal => SplineMode::B,
            SplineMode::B => SplineMode::Hermite,
        }
    }

    // C => change cycling mode
    if keyboard.just_pressed(KeyCode::KeyC) {
        *cycling_mode = match *cycling_mode {
            CyclingMode::NotCyclic => CyclingMode::Cyclic,
            CyclingMode::Cyclic => CyclingMode::NotCyclic,
        }
    }

    // R => remove last control point
    if keyboard.just_pressed(KeyCode::KeyR) {
        control_points.points_and_tangents.pop();
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        std::process::exit(0);
    }
}

fn curve_fill(
    par_commands: ParallelCommands,
    curves: Query<(Entity, &Curve), (With<Curve>, Without<ProcessedCurve>)>,
) {
    curves.par_iter().for_each(|(id, curve)| {
        let resolution = 9 * curve.0.segments().len();
        let points: Vec<_> = curve.0.iter_positions(resolution).collect();
        par_commands.command_scope(|mut commands| {
            commands
                .entity(id)
                .insert(ProcessedCurve { interp: points });
        })
    });
}
