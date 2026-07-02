use bevy::{
    ecs::{query::ROQueryItem, system::SystemParamItem},
    prelude::*,
};
use yanor_core::grid::GridPos;

use crate::{BlockHandles, HandleProvider, presentation::IntoPresentableBundle};

pub trait Block: Component {}

#[derive(Component)]
pub struct DirtBlock;
impl Block for DirtBlock {}

#[derive(Component)]
pub struct GrassBlock;
impl Block for GrassBlock {}

#[derive(Component)]
pub struct PathBlock;
impl Block for PathBlock {}

#[derive(Component)]
pub struct StoneBlock;
impl Block for StoneBlock {}
