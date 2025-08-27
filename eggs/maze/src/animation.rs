use std::{collections::VecDeque, time::Duration};

use bevy::{prelude::*, transform::commands};
use bevy_tweening::{*, lens::TransformPositionLens};
use yanor_core::{
    grid::GridPosition,
    tick::{PendingPostTick, TickState, Tickable},
};

// TODO: better name
pub struct AnimationPresenterPlugin;

impl Plugin for AnimationPresenterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin)
            .init_resource::<AnimationConfig>()
            .init_resource::<AnimationQueue>()
            .add_systems(OnEnter(TickState::PostTick), (start_animating, finish_post_tick_if_no_animation))
            // .add_systems(OnEnter(AnimatingAll), start_all_animations)
            // .add_systems(OnEnter(AnimatingSequential), start_next_animation)
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
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    mut anim_queue: ResMut<AnimationQueue>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    info!("henlo");

    if !anim_queue.is_empty() {
        if anim_config.sequential_animations {
            // next_anim_state.set(AnimationState::AnimatingSequential);
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
    no_animation_query: Query<Entity, (With<Tickable>, With<PendingPostTick>, Without<PendingAnimation>)>,
) {
    for entity in no_animation_query {
        commands
            .entity(entity)
            .remove::<PendingPostTick>();
    }
}

fn start_all_animations(
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    info!("1");

    for (entity, pending_animation, transform, grid_pos) in pending_animation_query {
        info!("ooga booga");

        commands
            .entity(entity)
            .remove::<PendingAnimation>()
            .insert(create_step_animator(anim_config.animation_length, (pending_animation, transform, grid_pos)));
    }
}

fn start_next_animation(
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    mut anim_queue: ResMut<AnimationQueue>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    info!("2");

    if let Some(entity) = anim_queue.pop_front() {
        info!("next in q");

        if let Ok((_, pending_animation, transform, grid_pos)) = pending_animation_query.get(entity) {
            commands
                .entity(entity)
                .remove::<PendingAnimation>()
                .insert(create_step_animator(anim_config.animation_length, (pending_animation, transform, grid_pos)));
        } else {
            warn!("Entity in animation queue not found in query (possibly missing needed components)");
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

fn queue_step_animation_observer(
    trigger: Trigger<OnReplace, GridPosition>,
    mut commands: Commands,
    mut anim_queue: ResMut<AnimationQueue>,
) {
    info!("pend");

    let entity = trigger.target();
    anim_queue.push_back(entity);
    commands.entity(entity).insert(PendingAnimation::Step);
}

fn on_tween_completed(
    trigger: Trigger<TweenCompleted>,
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    mut anim_queue: ResMut<AnimationQueue>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    info!("tc");

    commands
        .entity(trigger.target())
        .remove::<Animator<Transform>>()
        .remove::<PendingPostTick>();

    if anim_config.sequential_animations {
        start_next_animation(commands, anim_config, anim_queue, pending_animation_query);
    }
}
