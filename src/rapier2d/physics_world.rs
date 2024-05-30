use crate::rapier2d::convert::meters_to_pixels;
use crate::rapier2d::convert::pixels_to_meters;
use crate::rapier2d::convert::vector_meters_to_pixels;
use crate::rapier2d::convert::vector_pixels_to_meters;
use crate::rapier2d::event_handler::ContactEventHandler;
use crate::rapier2d::handle::*;
use crate::rapier2d::physics_hooks::*;
use crate::rapier2d::settings::*;
use crate::rapier2d::user_data::*;
use crate::rapier2d::vector::Vector;
use godot::log::godot_error;
use rapier2d::crossbeam;
use rapier2d::data::Arena;
use rapier2d::prelude::*;
use salva2d::integrations::rapier::FluidsPipeline;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

pub struct ActiveBodyInfo {
    pub body_handle: Handle,
    pub body_user_data: UserData,
}

impl ActiveBodyInfo {
    fn new() -> ActiveBodyInfo {
        ActiveBodyInfo {
            body_handle: invalid_handle(),
            body_user_data: invalid_user_data(),
        }
    }
}

pub struct ContactPointInfo {
    pub pixel_local_pos_1: Vector,
    pub pixel_local_pos_2: Vector,
    pub pixel_velocity_pos_1: Vector,
    pub pixel_velocity_pos_2: Vector,
    pub normal: Vector,
    pub pixel_distance: Real,
    pub pixel_impulse: Real,
    pub pixel_tangent_impulse: Real,
}

impl ContactPointInfo {
    fn new() -> ContactPointInfo {
        ContactPointInfo {
            pixel_local_pos_1: Vector { x: 0.0, y: 0.0 },
            pixel_local_pos_2: Vector { x: 0.0, y: 0.0 },
            pixel_velocity_pos_1: Vector { x: 0.0, y: 0.0 },
            pixel_velocity_pos_2: Vector { x: 0.0, y: 0.0 },
            normal: Vector { x: 0.0, y: 0.0 },
            pixel_distance: 0.0,
            pixel_impulse: 0.0,
            pixel_tangent_impulse: 0.0,
        }
    }
}

type ActiveBodyCallback = fn(active_body_info: &ActiveBodyInfo);

type CollisionEventCallback = fn(world_handle: Handle, event_info: &CollisionEventInfo);

type ContactForceEventCallback =
    fn(world_handle: Handle, event_info: &ContactForceEventInfo) -> bool;
type ContactPointCallback = fn(
    world_handle: Handle,
    contact_info: &ContactPointInfo,
    event_info: &ContactForceEventInfo,
) -> bool;

pub struct CollisionEventInfo {
    pub collider1: Handle,
    pub collider2: Handle,
    pub user_data1: UserData,
    pub user_data2: UserData,
    pub is_sensor: bool,
    pub is_started: bool,
    pub is_stopped: bool,
    pub is_removed: bool,
}

pub struct ContactForceEventInfo {
    pub collider1: Handle,
    pub collider2: Handle,
    pub user_data1: UserData,
    pub user_data2: UserData,
}

#[derive(Serialize, Deserialize)]
pub struct PhysicsObjects {
    pub query_pipeline: QueryPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhaseMultiSap,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,

    pub collider_set: ColliderSet,
    pub rigid_body_set: RigidBodySet,

    pub handle: Handle,
}

pub struct PhysicsWorld {
    pub physics_objects: PhysicsObjects,
    pub physics_pipeline: PhysicsPipeline,
    pub fluids_pipeline: FluidsPipeline,
}

impl PhysicsWorld {
    pub fn new(settings: &WorldSettings) -> PhysicsWorld {
        PhysicsWorld {
            physics_objects: PhysicsObjects {
                query_pipeline: QueryPipeline::new(),
                island_manager: IslandManager::new(),
                broad_phase: DefaultBroadPhase::new(),
                narrow_phase: NarrowPhase::new(),
                impulse_joint_set: ImpulseJointSet::new(),
                multibody_joint_set: MultibodyJointSet::new(),
                ccd_solver: CCDSolver::new(),
    
                rigid_body_set: RigidBodySet::new(),
                collider_set: ColliderSet::new(),
    
                handle: invalid_handle(),
            },
            physics_pipeline: PhysicsPipeline::new(),
            fluids_pipeline: FluidsPipeline::new(
                pixels_to_meters(settings.particle_radius),
                settings.smoothing_factor,
            ),
        }
    }

    pub fn step(
        &mut self,
        settings: &SimulationSettings,
        active_body_callback: ActiveBodyCallback,
        collision_filter_body_callback: CollisionFilterCallback,
        collision_filter_sensor_callback: CollisionFilterCallback,
        collision_modify_contacts_callback: CollisionModifyContactsCallback,
        collision_event_callback: CollisionEventCallback,
        contact_force_event_callback: ContactForceEventCallback,
        contact_point_callback: ContactPointCallback,
    ) {
        let mut integration_parameters = IntegrationParameters::default();

        integration_parameters.dt = settings.dt;
        integration_parameters.max_ccd_substeps = settings.max_ccd_substeps;
        if settings.num_solver_iterations > 0 {
            integration_parameters.num_solver_iterations =
                NonZeroUsize::new(settings.num_solver_iterations).unwrap();
        }
        integration_parameters.num_additional_friction_iterations =
            settings.num_additional_friction_iterations;
        integration_parameters.num_internal_pgs_iterations = settings.num_internal_pgs_iterations;
        let gravity_vector = vector_pixels_to_meters(&settings.pixel_gravity);
        let gravity = vector![gravity_vector.x, gravity_vector.y];
        let liquid_gravity_vector = vector_pixels_to_meters(&settings.pixel_liquid_gravity);
        let liquid_gravity = vector![liquid_gravity_vector.x, liquid_gravity_vector.y];

        let physics_hooks = PhysicsHooksCollisionFilter {
            world_handle: self.physics_objects.handle,
            collision_filter_body_callback: &collision_filter_body_callback,
            collision_filter_sensor_callback: &collision_filter_sensor_callback,
            collision_modify_contacts_callback: &collision_modify_contacts_callback,
        };

        // Initialize the event collector.
        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
        let event_handler = ContactEventHandler::new(collision_send, contact_force_send);

        self.physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut self.physics_objects.island_manager,
            &mut self.physics_objects.broad_phase,
            &mut self.physics_objects.narrow_phase,
            &mut self.physics_objects.rigid_body_set,
            &mut self.physics_objects.collider_set,
            &mut self.physics_objects.impulse_joint_set,
            &mut self.physics_objects.multibody_joint_set,
            &mut self.physics_objects.ccd_solver,
            Some(&mut self.physics_objects.query_pipeline),
            &physics_hooks,
            &event_handler,
        );
        if self.fluids_pipeline.liquid_world.fluids().len() > 0 {
            self.fluids_pipeline.step(
                &liquid_gravity,
                integration_parameters.dt,
                &self.physics_objects.collider_set,
                &mut self.physics_objects.rigid_body_set,
            );
        }

        for handle in self.physics_objects.island_manager.active_dynamic_bodies() {
            // Send the active body event.
            let active_body_info = ActiveBodyInfo {
                body_handle: rigid_body_handle_to_handle(*handle),
                body_user_data: self.get_rigid_body_user_data(*handle),
            };

            (active_body_callback)(&active_body_info);
        }
        for handle in self.physics_objects.island_manager.active_kinematic_bodies() {
            // Send the active body event.
            let active_body_info = ActiveBodyInfo {
                body_handle: rigid_body_handle_to_handle(*handle),
                body_user_data: self.get_rigid_body_user_data(*handle),
            };

            (active_body_callback)(&active_body_info);
        }

        while let Ok((collision_event, _contact_pair)) = collision_recv.try_recv() {
            let handle1 = collision_event.collider1();
            let handle2 = collision_event.collider2();

            // Handle the collision event.
            let event_info = CollisionEventInfo {
                is_sensor: collision_event.sensor(),
                is_removed: collision_event.removed(),
                is_started: collision_event.started(),
                is_stopped: collision_event.stopped(),
                collider1: collider_handle_to_handle(handle1),
                collider2: collider_handle_to_handle(handle2),
                user_data1: self.get_collider_user_data(handle1),
                user_data2: self.get_collider_user_data(handle2),
            };

            (collision_event_callback)(self.physics_objects.handle, &event_info);
        }

        while let Ok((contact_force_event, contact_pair)) = contact_force_recv.try_recv() {
            let collider1 = self
                .physics_objects
                .collider_set
                .get(contact_force_event.collider1)
                .unwrap();
            let collider2 = self
            .physics_objects.collider_set
                .get(contact_force_event.collider2)
                .unwrap();

            // Handle the contact force event.
            let event_info = ContactForceEventInfo {
                collider1: collider_handle_to_handle(contact_force_event.collider1),
                collider2: collider_handle_to_handle(contact_force_event.collider2),
                user_data1: UserData::new(collider1.user_data),
                user_data2: UserData::new(collider2.user_data),
            };

            let mut send_contact_points = (contact_force_event_callback)(self.physics_objects.handle, &event_info);

            if send_contact_points {
                let body1: &RigidBody = self.get_collider_rigid_body(collider1).unwrap();
                let body2: &RigidBody = self.get_collider_rigid_body(collider2).unwrap();
                // Find the contact pair, if it exists, between two colliders
                let mut contact_info = ContactPointInfo::new();

                let mut swap = false;
                if contact_force_event.collider1 != contact_pair.collider1 {
                    assert!(contact_force_event.collider1 == contact_pair.collider2);
                    assert!(contact_force_event.collider2 == contact_pair.collider1);
                    swap = true;
                } else {
                    assert!(contact_force_event.collider2 == contact_pair.collider2);
                }

                // We may also read the contact manifolds to access the contact geometry.
                for manifold in &contact_pair.manifolds {
                    let manifold_normal = manifold.data.normal;
                    contact_info.normal = Vector {
                        x: manifold_normal.x,
                        y: manifold_normal.y,
                    };

                    // Read the geometric contacts.
                    for contact_point in &manifold.points {
                        let collider_pos_1 = collider1.position() * contact_point.local_p1;
                        let collider_pos_2 = collider2.position() * contact_point.local_p2;
                        let point_velocity_1 = body1.velocity_at_point(&collider_pos_1);
                        let point_velocity_2 = body2.velocity_at_point(&collider_pos_2);

                        if swap {
                            contact_info.pixel_local_pos_1 = vector_meters_to_pixels(&Vector {
                                x: collider_pos_2.x,
                                y: collider_pos_2.y,
                            });
                            contact_info.pixel_local_pos_2 = vector_meters_to_pixels(&Vector {
                                x: collider_pos_1.x,
                                y: collider_pos_1.y,
                            });
                            contact_info.pixel_velocity_pos_1 = vector_meters_to_pixels(&Vector {
                                x: point_velocity_2.x,
                                y: point_velocity_2.y,
                            });
                            contact_info.pixel_velocity_pos_2 = vector_meters_to_pixels(&Vector {
                                x: point_velocity_1.x,
                                y: point_velocity_1.y,
                            });
                        } else {
                            contact_info.pixel_local_pos_1 = vector_meters_to_pixels(&Vector {
                                x: collider_pos_1.x,
                                y: collider_pos_1.y,
                            });
                            contact_info.pixel_local_pos_2 = vector_meters_to_pixels(&Vector {
                                x: collider_pos_2.x,
                                y: collider_pos_2.y,
                            });
                            contact_info.pixel_velocity_pos_1 = vector_meters_to_pixels(&Vector {
                                x: point_velocity_1.x,
                                y: point_velocity_1.y,
                            });
                            contact_info.pixel_velocity_pos_2 = vector_meters_to_pixels(&Vector {
                                x: point_velocity_2.x,
                                y: point_velocity_2.y,
                            });
                        }
                        contact_info.pixel_distance = meters_to_pixels(contact_point.dist);
                        contact_info.pixel_impulse = meters_to_pixels(contact_point.data.impulse);
                        contact_info.pixel_tangent_impulse =
                            meters_to_pixels(contact_point.data.tangent_impulse.x);

                        send_contact_points =
                            (contact_point_callback)(self.physics_objects.handle, &contact_info, &event_info);
                        if !send_contact_points {
                            break;
                        }
                    }

                    if !send_contact_points {
                        break;
                    }
                }
            }
        }
    }

    pub fn insert_collider(&mut self, collider: Collider, body_handle: Handle) -> Handle {
        if body_handle.is_valid() {
            let rigid_body_handle = handle_to_rigid_body_handle(body_handle);
            let collider_handle = self.physics_objects.collider_set.insert_with_parent(
                collider,
                rigid_body_handle,
                &mut self.physics_objects.rigid_body_set,
            );
            collider_handle_to_handle(collider_handle)
        } else {
            let collider_handle = self.physics_objects.collider_set.insert(collider);
            collider_handle_to_handle(collider_handle)
        }
    }

    pub fn remove_collider(&mut self, handle: Handle) {
        let collider_handle = handle_to_collider_handle(handle);
        self.physics_objects.collider_set.remove(
            collider_handle,
            &mut self.physics_objects.island_manager,
            &mut self.physics_objects.rigid_body_set,
            false,
        );
    }

    pub fn get_collider_user_data(&self, collider_handle: ColliderHandle) -> UserData {
        let collider = self.physics_objects.collider_set.get(collider_handle);
        if collider.is_none() {
            invalid_user_data()
        } else {
            UserData::new(collider.unwrap().user_data)
        }
    }

    pub fn get_collider_rigid_body(&self, collider: &Collider) -> Option<&RigidBody> {
        let parent = collider.parent();
        if parent.is_some() {
            return self.physics_objects.rigid_body_set.get(parent.unwrap());
        } else {
            None
        }
    }

    pub fn remove_rigid_body(&mut self, body_handle: Handle) {
        let rigid_body_handle = handle_to_rigid_body_handle(body_handle);
        self.physics_objects.rigid_body_set.remove(
            rigid_body_handle,
            &mut self.physics_objects.island_manager,
            &mut self.physics_objects.collider_set,
            &mut self.physics_objects.impulse_joint_set,
            &mut self.physics_objects.multibody_joint_set,
            true,
        );
    }

    pub fn get_rigid_body_user_data(&self, rigid_body_handle: RigidBodyHandle) -> UserData {
        let rigid_body = self.physics_objects.rigid_body_set.get(rigid_body_handle);
        if rigid_body.is_none() {
            invalid_user_data()
        } else {
            UserData::new(rigid_body.unwrap().user_data)
        }
    }

    pub fn insert_joint(
        &mut self,
        body_handle_1: Handle,
        body_handle_2: Handle,
        joint: impl Into<GenericJoint>,
    ) -> Handle {
        let rigid_body_1_handle = handle_to_rigid_body_handle(body_handle_1);
        let rigid_body_2_handle = handle_to_rigid_body_handle(body_handle_2);

        let joint_handle =
            self.physics_objects.impulse_joint_set
                .insert(rigid_body_1_handle, rigid_body_2_handle, joint, true);
        joint_handle_to_handle(joint_handle)
    }

    pub fn remove_joint(&mut self, handle: Handle) {
        let joint_handle = handle_to_joint_handle(handle);
        self.physics_objects.impulse_joint_set.remove(joint_handle, true);
    }
}

pub struct PhysicsEngine {
    pub physics_worlds: Arena<PhysicsWorld>,
    pub shapes: Arena<SharedShape>,
}

impl PhysicsEngine {
    fn new() -> PhysicsEngine {
        PhysicsEngine {
            physics_worlds: Arena::new(),
            shapes: Arena::new(),
        }
    }

    pub fn insert_world(&mut self, world: PhysicsWorld) -> Handle {
        let world_handle = self.physics_worlds.insert(world);
        world_handle_to_handle(world_handle)
    }

    pub fn remove_world(&mut self, handle: Handle) {
        let world_handle = handle_to_world_handle(handle);
        self.physics_worlds.remove(world_handle);
    }

    pub fn get_world(&mut self, handle: Handle) -> &mut PhysicsWorld {
        let world_handle = handle_to_world_handle(handle);
        let world = self.physics_worlds.get_mut(world_handle);
        assert!(world.is_some());
        world.unwrap()
    }

    pub fn insert_shape(&mut self, shape: SharedShape) -> Handle {
        let shape_handle = self.shapes.insert(shape);
        shape_handle_to_handle(shape_handle)
    }

    pub fn remove_shape(&mut self, handle: Handle) {
        let shape_handle = handle_to_shape_handle(handle);
        self.shapes.remove(shape_handle);
    }

    pub fn get_shape(&mut self, handle: Handle) -> &SharedShape {
        let shape_handle = handle_to_shape_handle(handle);
        let shape = self.shapes.get(shape_handle);
        assert!(shape.is_some());
        shape.unwrap()
    }
}

pub fn singleton() -> &'static mut PhysicsEngine {
    static mut SINGLETON: Option<PhysicsEngine> = None;
    unsafe {
        if SINGLETON.is_none() {
            SINGLETON = Some(PhysicsEngine::new());
        }
        SINGLETON.as_mut().unwrap()
    }
}

pub fn world_create(settings: &WorldSettings) -> Handle {
    let physics_world = PhysicsWorld::new(settings);
    let physics_engine = singleton();
    let world_handle = physics_engine.insert_world(physics_world);
    let physics_world = physics_engine.get_world(world_handle);
    physics_world.physics_objects.handle = world_handle;
    world_handle
}

pub fn world_destroy(world_handle: Handle) {
    let physics_engine = singleton();
    let physics_world = physics_engine.get_world(world_handle);
    physics_world.physics_objects.handle = invalid_handle();
    physics_engine.remove_world(world_handle);
}

pub fn world_step(
    world_handle: Handle,
    settings: &SimulationSettings,
    active_body_callback: ActiveBodyCallback,
    collision_filter_body_callback: CollisionFilterCallback,
    collision_filter_sensor_callback: CollisionFilterCallback,
    collision_modify_contacts_callback: CollisionModifyContactsCallback,
    collision_event_callback: CollisionEventCallback,
    contact_force_event_callback: ContactForceEventCallback,
    contact_point_callback: ContactPointCallback,
) {
    let physics_engine = singleton();
    let physics_world = physics_engine.get_world(world_handle);
    physics_world.step(
        settings,
        active_body_callback,
        collision_filter_body_callback,
        collision_filter_sensor_callback,
        collision_modify_contacts_callback,
        collision_event_callback,
        contact_force_event_callback,
        contact_point_callback,
    );
}

pub fn world_get_active_objects_count(world_handle: Handle) -> usize {
    let physics_engine = singleton();
    let physics_world = physics_engine.get_world(world_handle);
    return physics_world.physics_objects.island_manager.active_dynamic_bodies().len();
}

pub fn world_export_json(world_handle: Handle) -> String {
    let physics_engine = singleton();
    let physics_world = physics_engine.get_world(world_handle);
    let serialized = serde_json::to_string(&physics_world.physics_objects.collider_set);
    match serialized {
        Ok(serialized) => {
            serialized
        },
        Err(err) => {
            godot_error!("{}", err);
            String::from("{}")
        },
    }
}

pub fn world_export_binary(world_handle: Handle) -> Vec<u8> {
    let physics_engine = singleton();
    let physics_world = physics_engine.get_world(world_handle);
    let serialized = bincode::serialize(&physics_world.physics_objects);
    match serialized {
        Ok(serialized) => {
            serialized
        },
        Err(err) => {
            godot_error!("{}", err);
            Vec::new()
        },
    }
}
