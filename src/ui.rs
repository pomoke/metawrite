use bevy::{
    app::{App, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    prelude::*,
    state::state::{NextState, State, States},
};

#[derive(Event, Debug, Clone)]
pub enum OverlayEvent {
    Normal,
    Overlay(String),
    /// TODO: Message goes off after some time.
    Transient(String),
}

#[derive(States, Debug, Clone, Hash, Eq, PartialEq)]
pub enum OverlayState {
    Normal,
    Blocked,
}

#[derive(Component, Debug, Clone)]
pub struct OverlayMarker;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_overlay,));
    }
}

fn draw_overlay(
    mut commands: Commands,
    state: Res<State<OverlayState>>,
    mut events: EventReader<OverlayEvent>,
    mut next_state: ResMut<NextState<OverlayState>>,
    overlay: Query<Entity, With<OverlayMarker>>,
) {
    overlay
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
    for i in events.read() {
        match i {
            OverlayEvent::Normal => {
                *next_state = NextState::Pending(OverlayState::Normal);
            }
            OverlayEvent::Overlay(msg) => {
                *next_state = NextState::Pending(OverlayState::Blocked);

                commands
                    .spawn((
                        OverlayMarker,
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            top: Val::Px(0.),
                            left: Val::Px(0.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(Val::Px(20.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                BorderRadius::all(Val::Px(10.0)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new("Loading..."),
                                    TextFont {
                                        font_size: 24.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
            }
            _ => {}
        }
    }
}
