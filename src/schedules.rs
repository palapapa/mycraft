use bevy_ecs::schedule::*;
use crate::systems::egui::*;
use crate::systems::transform::*;

const COMMON_SCHEDULE_BUILD_SETTINGS: ScheduleBuildSettings = ScheduleBuildSettings {
    ambiguity_detection: LogLevel::Warn,
    auto_insert_apply_deferred: true,
    hierarchy_detection: LogLevel::Warn,
    report_sets: true,
    use_shortnames: true
};

/// The [`Schedule`] that runs every frame.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpdateSchedule;

impl UpdateSchedule {
    /// Creates a [`Schedule`] that uses this `struct` as a label and configures
    /// systems and build settings.
    pub fn create_schedule() -> Schedule {
        let mut schedule = Schedule::new(Self);
        schedule.set_build_settings(COMMON_SCHEDULE_BUILD_SETTINGS);
        schedule
    }
}

/// The [`Schedule`] that is only ever run once when the app starts.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StartupSchedule;

impl StartupSchedule {
    /// Creates a [`Schedule`] that uses this `struct` as a label and configures
    /// systems and build settings.
    pub fn create_schedule() -> Schedule {
        let mut schedule = Schedule::new(Self);
        schedule.set_build_settings(COMMON_SCHEDULE_BUILD_SETTINGS)
            .add_systems(initialize_egui_system);
        schedule
    }
}

/// The [`Schedule`] that runs after [`UpdateSchedule`].
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostUpdateSchedule;

impl PostUpdateSchedule {
    pub fn create_schedule() -> Schedule {
        let mut schedule = Schedule::new(Self);
        schedule.set_build_settings(COMMON_SCHEDULE_BUILD_SETTINGS)
            .add_systems((mark_dirty_trees_system, propagate_parent_transforms_system).chain());
        schedule
    }
}

/// The [`Schedule`] that renders the scene. It is run after [`PostUpdateSchedule`].
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderSchedule;

impl RenderSchedule {
    pub fn create_schedule() -> Schedule {
        let mut schedule = Schedule::new(Self);
        schedule.set_build_settings(COMMON_SCHEDULE_BUILD_SETTINGS)
            .add_systems(render_egui_system);
        schedule
    }
}
