use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{*, lens::TransformPositionLens};
use yanor_core::{
    grid::GridPosition,
    tick::TickState,
};

// TODO: better name
pub struct AnimateMovementPlugin;

impl Plugin for AnimateMovementPlugin {
    fn build(&self, app: &mut App) {
        use AnimationState::*;

        app.add_plugins(TweeningPlugin)
            .init_state::<AnimationState>()
            .init_resource::<AnimationConfig>()
            .init_resource::<AnimationQueue>()
            .add_systems(OnExit(TickState::PostTick), check_for_pending_animations)
            .add_systems(OnEnter(AnimatingAll), start_all_animations)
            .add_systems(OnEnter(AnimatingSequential), start_next_animation)
            .add_systems(
                FixedUpdate,
                skip_animation_on_input.run_if(not(in_state(NotAnimating))),
            )
            .add_observer(queue_step_animation_observer)
            .add_observer(on_tween_completed);
    }
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AnimationState {
    #[default]
    NotAnimating,
    AnimatingAll,
    AnimatingSequential,
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

fn check_for_pending_animations(
    anim_config: Res<AnimationConfig>,
    mut anim_queue: ResMut<AnimationQueue>,
    mut next_anim_state: ResMut<NextState<AnimationState>>,
) {
    if !anim_queue.is_empty() {
        // todo!("Pause ticking");

        if anim_config.sequential_animations {
            next_anim_state.set(AnimationState::AnimatingSequential);
        } else {
            // For non-sequential animation the queue is not necessary so go ahead and clear it
            anim_queue.clear();
            next_anim_state.set(AnimationState::AnimatingAll);
        }
    }
}

fn start_all_animations(
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    pending_animation_query: Query<(Entity, &PendingAnimation, &Transform, &GridPosition)>,
) {
    info!("all");

    for (entity, pending_animation, transform, grid_pos) in pending_animation_query {
        commands
            .entity(entity)
            .remove::<PendingAnimation>()
            .insert(create_animator(anim_config.animation_length, (pending_animation, transform, grid_pos)));
    }
}

fn start_next_animation(
    mut commands: Commands,
    anim_config: Res<AnimationConfig>,
    mut anim_queue: ResMut<AnimationQueue>,
    mut next_anim_state: ResMut<NextState<AnimationState>>,
    pending_animation_query: Query<(&PendingAnimation, &Transform, &GridPosition)>,
) {
    if let Some(entity) = anim_queue.pop_front() {
        info!("next in q");

        if let Ok(anim_components) = pending_animation_query.get(entity) {
            commands
                .entity(entity)
                .remove::<PendingAnimation>()
                .insert(create_animator(anim_config.animation_length, anim_components));
        } else {
            warn!("Entity in animation queue not found in query (possibly missing needed components)");
        }
    } else {
        info!("q empty");
        next_anim_state.set(AnimationState::NotAnimating);
    }
}

fn create_animator(
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
            animator.tweenable_mut().set_progress(0.99);
        }
    }
}

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
    mut anim_queue: ResMut<AnimationQueue>,
    current_anim_state: Res<State<AnimationState>>,
    mut next_anim_state: ResMut<NextState<AnimationState>>,
    pending_animation_query: Query<(&PendingAnimation, &Transform, &GridPosition)>,
) {
    info!("tc");

    commands
        .entity(trigger.target())
        .remove::<Animator<Transform>>();

    match current_anim_state.get() {
        AnimationState::AnimatingAll => next_anim_state.set(AnimationState::NotAnimating),
        AnimationState::AnimatingSequential => start_next_animation(commands, anim_config, anim_queue, next_anim_state, pending_animation_query),
        AnimationState::NotAnimating => warn!("TweenCompleted fired while in NotAnimating state"),
    };
}
