use std::collections::HashMap;

use bevy::{
    app::{App, Plugin},
    asset::AssetApp,
    ecs::{
        event::{Event, EventReader, EventWriter},
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    log::{debug, info},
    prelude::*,
    reflect::Reflect,
    state::state::States,
    tasks::Task,
};
use bevy_pkv::PkvStore;
use serde::{Deserialize, Serialize};

use crate::{Curve, ui::OverlayEvent};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub title: String,
    pub info: ProjectInfo,

    /// Canvas. Main canvas has name of `.main`.
    /// Any name begin with `.` is reserved for internal use.
    pub canvas: HashMap<String, Canvas>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct ProjectInfo {
    pub author: String,
    pub version: String,
    pub date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct Canvas {
    pub strokes: Vec<Curve>,
    pub elements: Vec<Elements>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub enum Elements {
    Curve(Curve),
    Peek(String),
    // TODO
    Shape(),
}


#[derive(Component)]
struct LoadTask(Task<Result<Project, String>>);

#[derive(Component)]
struct SaveTask(Task<Result<(), String>>);

pub fn do_save(
    curves: Query<&Curve>,
    overlay_event: EventWriter<OverlayEvent>,
    pkv: Res<PkvStore>,
) {
}

pub fn do_load(commands: Commands, pkv: Res<PkvStore>) {}
