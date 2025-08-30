use bevy_ecs::schedule::*;
use crate::system_sets::*;
use crate::systems::egui::*;

/// The [`Schedule`] that runs every frame.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpdateSchedule;

impl UpdateSchedule {
    /// Creates a [`Schedule`] that uses this `struct` as a label and configures
    /// systems and build settings.
    pub fn create_schedule() -> Schedule {
        let mut schedule = Schedule::new(Self);
        schedule.set_build_settings(ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, ..Default::default() })
            .add_systems(render_egui_system.in_set(EguiSystemSet));
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
        schedule.set_build_settings(ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, ..Default::default() })
            .add_systems(initialize_egui_system.in_set(EguiSystemSet));
        schedule
    }
}
