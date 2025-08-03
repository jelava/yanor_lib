use std::{collections::VecDeque, default, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{
    Animator, RepeatCount, Tween, TweenCompleted, TweeningPlugin, lens::TransformPositionLens,
};
use yanor_core::{
    activity::{ActivityPhase, BeginActivityPhase, FinishActivityPhase},
    grid::GridPosition,
    tick::TickState,
};

// TODO: better name
pub struct AnimateMovementPlugin;

impl Plugin for AnimateMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin)
            .init_state::<AnimationState>()
            .init_resource::<AnimationConfig>()
            .init_resource::<AnimationQueue>()
            // .init_resource::<AnimationTimer>()
            .add_systems(OnExit(TickState::PostTick), check_for_pending_animations)
            .add_systems(OnEnter(AnimationState::AnimatingAll), start_all_animations)
            .add_systems(
                FixedUpdate,
                skip_animation_on_input.run_if(not(in_state(AnimationState::NotAnimating))),
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
            animation_length: Duration::from_secs_f32(0.2),
        }
    }
}

// #[derive(Resource, Deref, DerefMut)]
// struct AnimationTimer(Timer);
//
// impl FromWorld for AnimationTimer {
//     fn from_world(world: &mut World) -> Self {
//         let config = world
//             .get_resource::<AnimationConfig>()
//             .expect("AnimationConfig resource must be initialized before AnimationTimer");
//
//         Self(Timer::from_seconds(config.animation_length, TimerMode::Once))
//     }
// }

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
    for (entity, pending_animation, transform, GridPosition(grid_pos)) in pending_animation_query {
        let start = transform.translation;
        let end = Vec3::new(grid_pos.x as f32, grid_pos.y as f32, grid_pos.z as f32);

        let tween = match pending_animation {
            PendingAnimation::Step => Tween::new(
                EaseFunction::QuadraticInOut,
                anim_config.animation_length,
                TransformPositionLens { start, end },
            )
            .with_completed_event(0),
        };

        commands
            .entity(entity)
            .remove::<PendingAnimation>()
            .insert(Animator::new(tween));
    }
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
    mut anim_queue: ResMut<AnimationQueue>,
    mut next_anim_state: ResMut<NextState<AnimationState>>,
) {
    commands
        .entity(trigger.target())
        .remove::<Animator<Transform>>();

    if let Some(_entity) = anim_queue.pop_front() {
        todo!("sequential anim stuff");
    } else {
        next_anim_state.set(AnimationState::NotAnimating);
    }
}
