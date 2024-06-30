#![feature(map_many_mut)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![feature(trait_alias)]
#[cfg(all(feature = "single", feature = "dim2"))]
extern crate rapier2d as rapier;
#[cfg(all(feature = "double", feature = "dim2"))]
extern crate rapier2d_f64 as rapier;
#[cfg(all(feature = "single", feature = "dim3"))]
extern crate rapier3d as rapier;
#[cfg(all(feature = "double", feature = "dim3"))]
extern crate rapier3d_f64 as rapier;
#[cfg(all(feature = "single", feature = "dim2"))]
extern crate salva2d as salva;
#[cfg(all(feature = "double", feature = "dim2"))]
extern crate salva2d as salva;
#[cfg(all(feature = "single", feature = "dim3"))]
extern crate salva3d as salva;
#[cfg(all(feature = "double", feature = "dim3"))]
extern crate salva3d as salva;
mod bodies;
mod fluids;
mod joints;
mod rapier_wrapper;
mod servers;
mod shapes;
mod spaces;
mod types;
use godot::prelude::*;
#[derive(GodotClass)]
#[class(base=Object, init)]
pub struct RapierPhysicsExtensionLibrary {}
#[gdextension]
unsafe impl ExtensionLibrary for RapierPhysicsExtensionLibrary {
    fn min_level() -> InitLevel {
        InitLevel::Servers
    }

    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Scene => {
                servers::register_scene();
            }
            InitLevel::Servers => {
                servers::register_server();
            }
            InitLevel::Editor => {
                servers::register_editor();
            }
            _ => (),
        }
    }

    fn on_level_deinit(level: InitLevel) {
        match level {
            InitLevel::Scene => {
                servers::unregister_scene();
            }
            InitLevel::Servers => {
                servers::unregister_server();
            }
            InitLevel::Editor => {
                servers::unregister_editor();
            }
            _ => (),
        }
    }
}
