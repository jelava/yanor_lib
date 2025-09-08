// TODO? should this be submodule of tick? or should it stay its own thing until there is a more
// general visuals/presentation module?

use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

use crate::{
    grid::GridPosition,
    tick::{PendingPostTick, TickState, Tickable},
};

// TODO: better name
pub struct AnimateTickPlugin;

impl Plugin for AnimateTickPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin)
            .init_resource::<AnimationConfig>()
            .init_resource::<AnimationQueue>()
            .add_systems(
                OnEnter(TickState::PostTick),
                (start_animating, finish_post_tick_if_no_animation),
            )
            .add_systems(
                FixedUpdate,
                skip_animation_on_input.run_if(in_state(TickState::PostTick)),
            )
            .add_observer(queue_step_animation_observer)
            .add_observer(on_tween_completed);
    }
}

#[derive(Resource)]
struct AnimationConfig {
    sequential_animations: bool,
    animation_length: Duration,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            sequential_animations: false,
            animation_length: Duration::from_secs_f32(0.5),
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct AnimationQueue(VecDeque<Entity>);

#[derive(Component)]
enum PendingAnimation {
    Step,
}

fn start_animating(
    commands: Commands,
    anim_config: Res<AnimationConfig>,
    mut anim_queue: ResMut<AnimationQueue>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    if !anim_queue.is_empty() {
        if anim_config.sequential_animations {
            start_next_animation(commands, anim_config, anim_queue, pending_animation_query);
        } else {
            // For non-sequential animation the queue is not necessary so go ahead and clear it
            anim_queue.clear();
            start_all_animations(commands, anim_config, pending_animation_query);
        }
    }
}

fn finish_post_tick_if_no_animation(
    mut commands: Commands,
    no_animation_query: Query<
        Entity,
        (
            With<Tickable>,
            With<PendingPostTick>,
            Without<PendingAnimation>,
        ),
    >,
) {
    for entity in no_animation_query {
        commands.entity(entity).remove::<PendingPostTick>();
    }
}

fn start_all_animations(
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    for (entity, pending_animation, transform, grid_pos) in pending_animation_query {
        commands
            .entity(entity)
            .remove::<PendingAnimation>()
            .insert(create_step_animator(
                anim_config.animation_length,
                (pending_animation, transform, grid_pos),
            ));
    }
}

fn start_next_animation(
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    mut anim_queue: ResMut<AnimationQueue>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    if let Some(entity) = anim_queue.pop_front() {
        if let Ok((_, pending_animation, transform, grid_pos)) = pending_animation_query.get(entity)
        {
            commands
                .entity(entity)
                .remove::<PendingAnimation>()
                .insert(create_step_animator(
                    anim_config.animation_length,
                    (pending_animation, transform, grid_pos),
                ));
        } else {
            warn!(
                "Entity in animation queue not found in query (possibly missing needed components)"
            );
        }
    }
}

fn create_step_animator(
    animation_length: Duration,
    anim_components: (&PendingAnimation, &Transform, &GridPosition),
) -> Animator<Transform> {
    let (pending_animation, transform, GridPosition(grid_pos)) = anim_components;
    let start = transform.translation;
    let end = Vec3::new(grid_pos.x as f32, grid_pos.y as f32, grid_pos.z as f32);

    let tween = match pending_animation {
        PendingAnimation::Step => Tween::new(
            EaseFunction::QuadraticInOut,
            animation_length,
            TransformPositionLens { start, end },
        )
        .with_completed_event(0),
    };

    Animator::new(tween)
}

fn skip_animation_on_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animator_query: Query<&mut Animator<Transform>>,
) {
    if keyboard_input.get_just_pressed().next().is_some() {
        for mut animator in &mut animator_query {
            // TODO? change to 1.0? there might be reliability issues that way though, i forgor
            animator.tweenable_mut().set_progress(0.99);
        }
    }
}

// For sequential animation, preserving information about the order in which changes within the
// tick happened is useful, so queuing happens immediately via observer rather than waiting
// until PostTick to check for differences between the Transform and the GridPosition (for example)
fn queue_step_animation_observer(
    trigger: Trigger<OnReplace, GridPosition>,
    mut commands: Commands,
    mut anim_queue: ResMut<AnimationQueue>,
) {
    let entity = trigger.target();
    anim_queue.push_back(entity);
    commands.entity(entity).insert(PendingAnimation::Step);
}

fn on_tween_completed(
    trigger: Trigger<TweenCompleted>,
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    anim_queue: ResMut<AnimationQueue>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    commands
        .entity(trigger.target())
        .remove::<Animator<Transform>>()
        .remove::<PendingPostTick>();

    if anim_config.sequential_animations {
        start_next_animation(commands, anim_config, anim_queue, pending_animation_query);
    }
}
