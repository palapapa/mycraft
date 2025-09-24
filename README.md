# Mycraft

This is a simple 3D game made for me to learn the basics of graphics programming
and how to make a simple game engine from scratch. This project first builds an
abstraction layer over `wgpu` and then use that to render the game. The `egui`
crate is used to render a debug UI over the frame rendered by `wgpu`.

This project uses the `bevy_ecs` crate only for its Entity Component System
(ECS) implementation. This project does not use the Bevy engine.

## Project Structure

- `diagrams`: Contains diagrams that are referred to in the documentation to help describe concepts that are hard to put into words.
- `src`: The source code.
  - `components/*`: Contains ECS component definitions.
  - `resources/*`: Contains [`bevy_ecs` resource](https://bevy-cheatbook.github.io/programming/res.html) definitions.
  - `systems/*`: Contains ECS system definitions.
  - `application_handler.rs`: Contains the main loop of the game. It manages the GPU state, dispatches window events and more.
  - `asset.rs`: Contains code that manages assets. It uses the `assets_manager` crate to do its job.
  - `camera.rs`: Contains type definitions related to `CameraComponent`.
  - `egui_renderer.rs`: Contains code that renderers the `egui` UI. `src/systems/egui.rs` eventually calls into the code defined here.
  - `egui_state.rs`: Contains type definitions of the globally accessible state used by `egui` renderers so that states can be kept across frames.
  - `extensions.rs`: Contains definitions of "extension methods" similar to the concept of extension methods in C#.
  - `main.rs`: The entry point that sets up `env_logger` and calls into `application_handler.rs`.
  - `material.rs`: Contains the definition of `Material` that abstracts over shaders.
  - `mesh.rs`: Contains mesh-related definitions.
  - `schedules.rs`: Contains [`bevy_ecs` schedule](https://bevy-cheatbook.github.io/programming/schedules.html) definitions.
  - `shapes.rs`: Contains definitions of types that represent different shapes and methods that convert them into meshes.
  - `system_sets.rs`: Contains [`bevy_ecs` system set](https://bevy-cheatbook.github.io/programming/system-sets.html) definitions.
  - `world.rs`: Contains functions that create a [`bevy_ecs` world](https://bevy-cheatbook.github.io/programming/intro-data.html).
- `build.rs`: The build script that hard links the files inside the `assets` directory next to the built executable to make them accessible at runtime.
