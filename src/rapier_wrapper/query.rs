use std::ops::Mul;

use godot::log::godot_error;
use nalgebra::zero;
use rapier::parry;
use rapier::parry::query::ShapeCastOptions;
use rapier::prelude::*;

use crate::rapier_wrapper::prelude::*;
use crate::servers::rapier_physics_server_extra::PhysicsCollisionObjects;
use crate::spaces::rapier_space::RapierSpace;
pub struct RayHitInfo {
    pub pixel_position: Vector<Real>,
    pub normal: Vector<Real>,
    pub collider: ColliderHandle,
    pub user_data: UserData,
}
impl RayHitInfo {
    pub fn default() -> RayHitInfo {
        RayHitInfo {
            pixel_position: zero(),
            normal: zero(),
            collider: ColliderHandle::invalid(),
            user_data: UserData::invalid_user_data(),
        }
    }
}
#[derive(Copy, Clone, Default)]
pub struct PointHitInfo {
    pub collider: ColliderHandle,
    pub user_data: UserData,
}
#[derive(Default)]
pub struct ShapeCastResult {
    pub collided: bool,
    pub toi: Real,
    pub pixel_witness1: Vector<Real>,
    pub pixel_witness2: Vector<Real>,
    pub normal1: Vector<Real>,
    pub normal2: Vector<Real>,
    pub collider: ColliderHandle,
    pub user_data: UserData,
}
impl ShapeCastResult {
    fn new() -> ShapeCastResult {
        ShapeCastResult {
            collided: false,
            toi: 1.0,
            collider: ColliderHandle::invalid(),
            pixel_witness1: zero(),
            pixel_witness2: zero(),
            normal1: zero(),
            normal2: zero(),
            user_data: UserData::invalid_user_data(),
        }
    }
}
#[derive(Default)]
pub struct ContactResult {
    pub collided: bool,
    pub within_margin: bool,
    pub pixel_distance: Real,
    pub pixel_point1: Vector<Real>,
    pub pixel_point2: Vector<Real>,
    pub normal1: Vector<Real>,
    pub normal2: Vector<Real>,
}
#[derive(Default)]
pub struct QueryExcludedInfo {
    pub query_collision_layer_mask: u32,
    pub query_canvas_instance_id: u64,
    // Pointer to array of objects
    pub query_exclude: Vec<ColliderHandle>,
    pub query_exclude_size: usize,
    pub query_exclude_body: i64,
}
impl PhysicsEngine {
    pub fn intersect_ray(
        &self,
        world_handle: WorldHandle,
        from: Vector<Real>,
        dir: Vector<Real>,
        length: Real,
        collide_with_body: bool,
        collide_with_area: bool,
        hit_from_inside: bool,
        hit_info: &mut RayHitInfo,
        handle_excluded_info: &QueryExcludedInfo,
        physics_collision_objects: &PhysicsCollisionObjects,
        space: &RapierSpace,
    ) -> bool {
        let mut result = false;
        let Some(physics_world) = self.get_world(world_handle) else {
            return false;
        };
        let ray = Ray::new(Point { coords: from }, dir);
        let mut filter = QueryFilter::new();
        if !collide_with_body {
            filter = filter.exclude_solids();
        }
        if !collide_with_area {
            filter = filter.exclude_sensors();
        }
        let predicate = |handle: ColliderHandle, _collider: &Collider| -> bool {
            !space.is_handle_excluded_callback(
                handle,
                &physics_world.get_collider_user_data(handle),
                handle_excluded_info,
                physics_collision_objects,
            )
        };
        filter.predicate = Some(&predicate);
        let mut length_current = Real::MAX;
        physics_world
            .physics_objects
            .query_pipeline
            .intersections_with_ray(
                &physics_world.physics_objects.rigid_body_set,
                &physics_world.physics_objects.collider_set,
                &ray,
                length,
                true,
                filter,
                |handle, intersection| {
                    // Find closest intersection
                    if intersection.time_of_impact > length_current {
                        return true;
                    }
                    // Callback called on each collider hit by the ray.
                    if hit_from_inside || intersection.time_of_impact != 0.0 {
                        length_current = intersection.time_of_impact;
                        result = true;
                        let hit_point = ray.point_at(intersection.time_of_impact);
                        let hit_normal = intersection.normal;
                        hit_info.pixel_position = hit_point.coords;
                        hit_info.normal = hit_normal;
                        hit_info.collider = handle;
                        hit_info.user_data = physics_world.get_collider_user_data(handle);
                    }
                    true // Continue to search.
                },
            );
        result
    }

    pub fn intersect_point(
        &self,
        world_handle: WorldHandle,
        position: Vector<Real>,
        collide_with_body: bool,
        collide_with_area: bool,
        hit_info_array: *mut PointHitInfo,
        hit_info_length: usize,
        handle_excluded_info: &QueryExcludedInfo,
        physics_collision_objects: &PhysicsCollisionObjects,
        space: &RapierSpace,
    ) -> usize {
        let mut cpt_hit = 0;
        if hit_info_length <= 0 {
            return cpt_hit;
        }
        if let Some(physics_world) = self.get_world(world_handle) {
            let point = Point { coords: position };
            let mut filter = QueryFilter::new();
            if !collide_with_body {
                filter = filter.exclude_solids();
            }
            if !collide_with_area {
                filter = filter.exclude_sensors();
            }
            let predicate = |handle: ColliderHandle, _collider: &Collider| -> bool {
                !space.is_handle_excluded_callback(
                    handle,
                    &physics_world.get_collider_user_data(handle),
                    handle_excluded_info,
                    physics_collision_objects,
                )
            };
            filter.predicate = Some(&predicate);
            let hit_info_slice_opt;
            unsafe {
                hit_info_slice_opt = Some(std::slice::from_raw_parts_mut(
                    hit_info_array,
                    hit_info_length,
                ));
            }
            assert!(hit_info_slice_opt.is_some());
            let hit_info_slice = hit_info_slice_opt.unwrap();
            physics_world
                .physics_objects
                .query_pipeline
                .intersections_with_point(
                    &physics_world.physics_objects.rigid_body_set,
                    &physics_world.physics_objects.collider_set,
                    &point,
                    filter,
                    |handle| {
                        // Callback called on each collider hit by the ray.
                        hit_info_slice[cpt_hit].collider = handle;
                        hit_info_slice[cpt_hit].user_data =
                            physics_world.get_collider_user_data(handle);
                        cpt_hit += 1;
                        cpt_hit < hit_info_length // Continue to search collisions if we still have space for results.
                    },
                );
        }
        cpt_hit
    }

    pub fn shape_collide(
        &self,
        shape_vel1: Vector<Real>,
        shape_info1: ShapeInfo,
        shape_vel2: Vector<Real>,
        shape_info2: ShapeInfo,
    ) -> ShapeCastResult {
        let mut shape_vel1 = shape_vel1;
        if shape_vel1 == Vector::zeros() {
            shape_vel1 = Vector::identity() * 1e-3;
        }
        let mut shape_vel2 = shape_vel2;
        if shape_vel2 == Vector::zeros() {
            shape_vel2 = Vector::identity() * 1e-3;
        }
        let mut result = ShapeCastResult::new();
        if let Some(raw_shared_shape1) = self.get_shape(shape_info1.handle) {
            let shared_shape1 = scale_shape(raw_shared_shape1, shape_info1);
            if let Some(raw_shared_shape2) = self.get_shape(shape_info2.handle) {
                let shared_shape2 = scale_shape(raw_shared_shape2, shape_info2);
                let shape_transform1 = shape_info1.transform;
                let shape_transform2 = shape_info2.transform;
                let mut shape_cast_options = ShapeCastOptions::default();
                shape_cast_options.max_time_of_impact = 1.0;
                shape_cast_options.compute_impact_geometry_on_penetration = true;
                shape_cast_options.stop_at_penetration = true;
                let toi_result = parry::query::cast_shapes(
                    &shape_transform1,
                    &shape_vel1,
                    shared_shape1.as_ref(),
                    &shape_transform2,
                    &shape_vel2,
                    shared_shape2.as_ref(),
                    shape_cast_options,
                );
                match toi_result {
                    Ok(None) => {}
                    Ok(Some(hit)) => {
                        result.collided = true;
                        result.toi = hit.time_of_impact;
                        result.normal1 = hit.normal1.into_inner();
                        result.normal2 = hit.normal2.into_inner();
                        result.pixel_witness1 = hit.witness1.coords;
                        result.pixel_witness2 = hit.witness2.coords;
                    }
                    Err(err) => {
                        godot_error!("toi error: {:?}", err);
                    }
                }
            }
        }
        result
    }

    pub fn shape_casting(
        &self,
        world_handle: WorldHandle,
        shape_vel: Vector<Real>,
        shape_info: ShapeInfo,
        collide_with_body: bool,
        collide_with_area: bool,
        handle_excluded_info: &QueryExcludedInfo,
        physics_collision_objects: &PhysicsCollisionObjects,
        space: &RapierSpace,
    ) -> ShapeCastResult {
        let mut shape_vel = shape_vel;
        if shape_vel == Vector::zeros() {
            shape_vel = Vector::identity() * 1e-3;
        }
        let mut result = ShapeCastResult::new();
        if let Some(raw_shared_shape) = self.get_shape(shape_info.handle) {
            let shared_shape = scale_shape(raw_shared_shape, shape_info);
            if let Some(physics_world) = self.get_world(world_handle) {
                let shape_transform = shape_info.transform;
                let mut filter = QueryFilter::new();
                if !collide_with_body {
                    filter = filter.exclude_solids();
                }
                if !collide_with_area {
                    filter = filter.exclude_sensors();
                }
                let predicate = |handle: ColliderHandle, _collider: &Collider| -> bool {
                    !space.is_handle_excluded_callback(
                        handle,
                        &physics_world.get_collider_user_data(handle),
                        handle_excluded_info,
                        physics_collision_objects,
                    )
                };
                filter.predicate = Some(&predicate);
                let mut shape_cast_options = ShapeCastOptions::default();
                //shape_cast_options.max_time_of_impact = Real::MAX;
                shape_cast_options.max_time_of_impact = 1.0;
                shape_cast_options.compute_impact_geometry_on_penetration = true;
                shape_cast_options.stop_at_penetration = true;
                shape_cast_options.target_distance = 0.0;
                if let Some((collider_handle, hit)) =
                    physics_world.physics_objects.query_pipeline.cast_shape(
                        &physics_world.physics_objects.rigid_body_set,
                        &physics_world.physics_objects.collider_set,
                        &shape_transform,
                        &shape_vel,
                        shared_shape.as_ref(),
                        shape_cast_options,
                        filter,
                    )
                {
                    result.collided = true;
                    result.toi = hit.time_of_impact;
                    result.normal1 = hit.normal1.into_inner();
                    result.normal2 = hit.normal2.into_inner();
                    result.collider = collider_handle;
                    result.user_data = physics_world.get_collider_user_data(collider_handle);
                    // first is in world space
                    let witness1 = hit.witness1;
                    // second is translated by collider transform
                    let mut witness2 = hit.witness2;
                    if let Some(collider) = physics_world
                        .physics_objects
                        .collider_set
                        .get(collider_handle)
                    {
                        witness2 += collider.position().translation.vector;
                    } else {
                        godot_error!("collider not found");
                    }
                    result.pixel_witness1 = witness1.coords;
                    result.pixel_witness2 = witness2.coords;
                }
            }
        }
        result
    }

    pub fn intersect_aabb(
        &self,
        world_handle: WorldHandle,
        aabb_min: Vector<Real>,
        aabb_max: Vector<Real>,
        collide_with_body: bool,
        collide_with_area: bool,
        hit_info_slice: &mut [PointHitInfo],
        max_results: usize,
        handle_excluded_info: &QueryExcludedInfo,
        physics_collision_objects: &PhysicsCollisionObjects,
        space: &RapierSpace,
    ) -> usize {
        let mut cpt_hit = 0;
        if let Some(physics_world) = self.get_world(world_handle) {
            let aabb_min_point = Point { coords: aabb_min };
            let aabb_max_point = Point { coords: aabb_max };
            let aabb = Aabb {
                mins: aabb_min_point,
                maxs: aabb_max_point,
            };
            physics_world
                .physics_objects
                .query_pipeline
                .colliders_with_aabb_intersecting_aabb(&aabb, |handle| {
                    let mut valid_hit = false;
                    if let Some(collider) = physics_world.physics_objects.collider_set.get(*handle)
                    {
                        // type filder
                        if collider.is_sensor() && collide_with_area {
                            valid_hit = true;
                        } else if !collider.is_sensor() && collide_with_body {
                            valid_hit = true;
                        }
                        if valid_hit {
                            valid_hit = !space.is_handle_excluded_callback(
                                *handle,
                                &physics_world.get_collider_user_data(*handle),
                                handle_excluded_info,
                                physics_collision_objects,
                            );
                        }
                    }
                    if !valid_hit {
                        return true; // continue
                    }
                    // Callback called on each collider hit by the ray.
                    hit_info_slice[cpt_hit].collider = *handle;
                    hit_info_slice[cpt_hit].user_data =
                        physics_world.get_collider_user_data(*handle);
                    cpt_hit += 1;
                    cpt_hit < max_results // Continue to search collisions if we still have space for results.
                });
        }
        cpt_hit
    }

    pub fn shapes_contact(
        &self,
        shape_info1: ShapeInfo,
        shape_info2: ShapeInfo,
        margin: Real,
    ) -> ContactResult {
        let mut result = ContactResult::default();
        //let prediction = Real::max(0.002, margin);
        let prediction = margin;
        if let Some(raw_shared_shape1) = self.get_shape(shape_info1.handle) {
            let shared_shape1 = scale_shape(raw_shared_shape1, shape_info1);
            if let Some(raw_shared_shape2) = self.get_shape(shape_info2.handle) {
                let shared_shape2 = scale_shape(raw_shared_shape2, shape_info2);
                let shape_transform1 = shape_info1.transform;
                let shape_transform2 = shape_info2.transform;
                if let Ok(Some(contact)) = parry::query::contact(
                    &shape_transform1,
                    shared_shape1.as_ref(),
                    &shape_transform2,
                    shared_shape2.as_ref(),
                    prediction,
                ) {
                    // the distance is negative if there is intersection
                    // and positive if the objects are separated by distance less than margin
                    result.pixel_distance = contact.dist;
                    result.within_margin = contact.dist > 0.0;
                    result.collided = true;
                    result.normal1 = contact.normal1.into_inner();
                    result.normal2 = contact.normal2.into_inner();
                    result.pixel_point1 = (contact.point1 + contact.normal1.mul(prediction)).coords;
                    result.pixel_point2 = contact.point2.coords;
                    return result;
                }
            }
        }
        result
    }
}
