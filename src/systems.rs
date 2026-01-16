#![expect(clippy::needless_pass_by_value, reason = "bevy_ecs requires that system parameters be passed by value.")]
#![expect(clippy::type_complexity, reason = "Query parameters often trigger thsi lint, but it is harmless.")]
pub mod egui;
pub mod transform;
