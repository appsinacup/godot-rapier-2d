use godot::classes::native::*;
use godot::prelude::*;

use super::rapier_space_body_helper::is_handle_excluded_callback;
use crate::bodies::rapier_collision_object::*;
use crate::bodies::vector_normalized;
use crate::rapier_wrapper::prelude::*;
use crate::servers::rapier_physics_server_extra::PhysicsData;
use crate::Angle;
use crate::Transform;
use crate::Vector;
#[cfg(feature = "dim2")]
type PhysicsServerExtensionShapeResult = PhysicsServer2DExtensionShapeResult;
#[cfg(feature = "dim2")]
type PhysicsServerExtensionRayResult = PhysicsServer2DExtensionRayResult;
#[cfg(feature = "dim2")]
type PhysicsServerExtensionShapeRestInfo = PhysicsServer2DExtensionShapeRestInfo;
#[cfg(feature = "dim3")]
type PhysicsServerExtensionShapeResult = PhysicsServer3DExtensionShapeResult;
#[cfg(feature = "dim3")]
type PhysicsServerExtensionRayResult = PhysicsServer3DExtensionRayResult;
#[cfg(feature = "dim3")]
type PhysicsServerExtensionShapeRestInfo = PhysicsServer3DExtensionShapeRestInfo;
pub struct RapierDirectSpaceStateImpl {
    pub space: Rid,
}
#[cfg(feature = "dim3")]
fn cross_product(a: Angle, b: Vector) -> Vector {
    a.cross(b)
}
#[cfg(feature = "dim2")]
fn cross_product(a: Angle, b: Vector) -> Vector {
    Vector::new(-a * b.y, a * b.x)
}
impl RapierDirectSpaceStateImpl {
    pub fn default() -> Self {
        Self {
            space: Rid::Invalid,
        }
    }

    pub unsafe fn intersect_ray(
        &mut self,
        from: Vector,
        to: Vector,
        collision_mask: u32,
        collide_with_bodies: bool,
        collide_with_areas: bool,
        hit_from_inside: bool,
        result: *mut PhysicsServerExtensionRayResult,
        physics_data: &PhysicsData,
    ) -> bool {
        let Some(space) = physics_data.spaces.get(&self.space) else {
            return false;
        };
        if !space.is_valid() {
            return false;
        }
        // Raycast Info
        let end = to - from;
        let dir = vector_normalized(end);
        let mut query_excluded_info = QueryExcludedInfo::default();
        query_excluded_info.query_collision_layer_mask = collision_mask;
        let mut hit_info = RayHitInfo::default();
        let collide = intersect_ray(
            space.get_handle(),
            vector_to_rapier(from),
            vector_to_rapier(dir),
            end.length(),
            collide_with_bodies,
            collide_with_areas,
            hit_from_inside,
            &mut hit_info,
            is_handle_excluded_callback,
            &query_excluded_info,
            physics_data,
        );
        if collide {
            let result = &mut *result;
            result.position = vector_to_godot(hit_info.pixel_position);
            result.normal = vector_to_godot(hit_info.normal);
            let (rid, shape_index) =
                RapierCollisionObject::get_collider_user_data(&hit_info.user_data);
            result.rid = rid;
            result.shape = shape_index as i32;
            if let Some(collision_object_2d) = physics_data.collision_objects.get(&result.rid) {
                let instance_id = collision_object_2d.get_base().get_instance_id();
                result.collider_id = ObjectId { id: instance_id };
                if instance_id != 0 {
                    if let Ok(object) =
                        Gd::<Node>::try_from_instance_id(InstanceId::from_i64(instance_id as i64))
                    {
                        result.set_collider(object)
                    }
                }
            }
            return true;
        }
        false
    }

    pub unsafe fn intersect_point(
        &mut self,
        position: Vector,
        canvas_instance_id: u64,
        collision_mask: u32,
        collide_with_bodies: bool,
        collide_with_areas: bool,
        results: *mut PhysicsServerExtensionShapeResult,
        max_results: i32,
        physics_data: &PhysicsData,
    ) -> i32 {
        let max_results = max_results as usize;
        if max_results <= 0 {
            return 0;
        }
        let Some(space) = physics_data.spaces.get(&self.space) else {
            return 0;
        };
        if !space.is_valid() {
            return 0;
        }
        // Allocate memory for hit_info_array
        let mut hit_info_array = Vec::new();
        hit_info_array.resize_with(max_results, Default::default);
        let hit_info_ptr = hit_info_array.as_mut_ptr();
        // Initialize query_excluded_info
        let mut query_excluded_info = QueryExcludedInfo::default();
        query_excluded_info.query_canvas_instance_id = canvas_instance_id;
        query_excluded_info.query_collision_layer_mask = collision_mask;
        // Perform intersection
        let mut result_count = intersect_point(
            space.get_handle(),
            vector_to_rapier(position),
            collide_with_bodies,
            collide_with_areas,
            hit_info_ptr,
            max_results,
            is_handle_excluded_callback,
            &mut query_excluded_info,
            physics_data,
        );
        if result_count > max_results {
            result_count = max_results;
        }
        let results_slice: &mut [PhysicsServerExtensionShapeResult] =
            unsafe { std::slice::from_raw_parts_mut(results, max_results) };
        for i in 0..max_results {
            let hit_info = unsafe { &mut *hit_info_ptr.add(i) };
            let (rid, shape_index) =
                RapierCollisionObject::get_collider_user_data(&hit_info.user_data);
            results_slice[i].rid = rid;
            results_slice[i].shape = shape_index as i32;
            let collision_object_2d = physics_data.collision_objects.get(&rid);
            if let Some(collision_object_2d) = collision_object_2d {
                let instance_id = collision_object_2d.get_base().get_instance_id();
                results_slice[i].collider_id = ObjectId { id: instance_id };
                if instance_id != 0 {
                    if let Ok(object) =
                        Gd::<Node>::try_from_instance_id(InstanceId::from_i64(instance_id as i64))
                    {
                        results_slice[i].set_collider(object)
                    }
                }
            }
        }
        result_count as i32
    }

    pub unsafe fn intersect_shape(
        &mut self,
        shape_rid: Rid,
        transform: Transform,
        motion: Vector,
        _margin: f32,
        collision_mask: u32,
        collide_with_bodies: bool,
        collide_with_areas: bool,
        results: *mut PhysicsServerExtensionShapeResult,
        max_results: i32,
        physics_data: &PhysicsData,
    ) -> i32 {
        let max_results = max_results as usize;
        let Some(shape) = physics_data.shapes.get(&shape_rid) else {
            return 0;
        };
        let Some(space) = physics_data.spaces.get(&self.space) else {
            return 0;
        };
        if !space.is_valid() {
            return 0;
        }
        let shape_info = shape_info_from_body_shape(shape.get_handle(), transform);
        let mut query_excluded_info = QueryExcludedInfo::default();
        query_excluded_info.query_collision_layer_mask = collision_mask;
        let mut query_exclude = Vec::new();
        query_exclude.resize_with(max_results, Default::default);
        query_excluded_info.query_exclude = query_exclude;
        query_excluded_info.query_exclude_size = 0;
        let mut cpt = 0;
        let results_slice: &mut [PhysicsServerExtensionShapeResult] =
            unsafe { std::slice::from_raw_parts_mut(results, max_results) };
        while cpt < max_results {
            let result = shape_casting(
                space.get_handle(),
                vector_to_rapier(motion),
                shape_info,
                collide_with_bodies,
                collide_with_areas,
                is_handle_excluded_callback,
                &query_excluded_info,
                physics_data,
            );
            if !result.collided {
                break;
            }
            query_excluded_info.query_exclude[query_excluded_info.query_exclude_size] =
                result.collider;
            query_excluded_info.query_exclude_size += 1;
            if !result.user_data.is_valid() {
                continue;
            }
            let (rid, shape_index) =
                RapierCollisionObject::get_collider_user_data(&result.user_data);
            let collision_object_2d = physics_data.collision_objects.get(&rid);
            if let Some(collision_object_2d) = collision_object_2d {
                results_slice[cpt].shape = shape_index as i32;
                results_slice[cpt].rid = rid;
                let instance_id = collision_object_2d.get_base().get_instance_id();
                results_slice[cpt].collider_id = ObjectId { id: instance_id };
                if instance_id != 0 {
                    if let Ok(object) =
                        Gd::<Node>::try_from_instance_id(InstanceId::from_i64(instance_id as i64))
                    {
                        results_slice[cpt].set_collider(object)
                    }
                }
                cpt += 1;
            }
        }
        cpt as i32
    }

    pub unsafe fn cast_motion(
        &mut self,
        shape_rid: Rid,
        transform: Transform,
        motion: Vector,
        _margin: f32,
        collision_mask: u32,
        collide_with_bodies: bool,
        collide_with_areas: bool,
        closest_safe: *mut f64,
        closest_unsafe: *mut f64,
        physics_data: &PhysicsData,
    ) -> bool {
        let Some(shape) = physics_data.shapes.get(&shape_rid) else {
            return false;
        };
        let Some(space) = physics_data.spaces.get(&self.space) else {
            return false;
        };
        if !space.is_valid() {
            return false;
        }
        let rapier_motion = vector_to_rapier(motion);
        let shape_info = shape_info_from_body_shape(shape.get_handle(), transform);
        let mut query_excluded_info = QueryExcludedInfo::default();
        query_excluded_info.query_collision_layer_mask = collision_mask;
        let hit = shape_casting(
            space.get_handle(),
            rapier_motion,
            shape_info,
            collide_with_bodies,
            collide_with_areas,
            is_handle_excluded_callback,
            &query_excluded_info,
            physics_data,
        )
        .toi;
        // TODO compute actual safe and unsafe
        let closest_safe = closest_safe as *mut real;
        *closest_safe = hit;
        let closest_unsafe = closest_unsafe as *mut real;
        *closest_unsafe = hit;
        true
    }

    pub unsafe fn collide_shape(
        &mut self,
        shape_rid: Rid,
        transform: Transform,
        motion: Vector,
        _margin: f32,
        collision_mask: u32,
        collide_with_bodies: bool,
        collide_with_areas: bool,
        results: *mut std::ffi::c_void,
        max_results: i32,
        result_count: *mut i32,
        physics_data: &PhysicsData,
    ) -> bool {
        let max_results = max_results as usize;
        let Some(shape) = physics_data.shapes.get(&shape_rid) else {
            return false;
        };
        let Some(space) = physics_data.spaces.get(&self.space) else {
            return false;
        };
        if !space.is_valid() {
            return false;
        }
        let results_out = results as *mut Vector;
        let shape_info = shape_info_from_body_shape(shape.get_handle(), transform);
        let mut query_excluded_info = QueryExcludedInfo::default();
        query_excluded_info.query_collision_layer_mask = collision_mask;
        let mut query_exclude = Vec::new();
        query_exclude.resize_with(max_results, Default::default);
        query_excluded_info.query_exclude = query_exclude;
        query_excluded_info.query_exclude_size = 0;
        let mut array_idx = 0;
        let mut cpt = 0;
        while cpt < max_results {
            let result = shape_casting(
                space.get_handle(),
                vector_to_rapier(motion),
                shape_info,
                collide_with_bodies,
                collide_with_areas,
                is_handle_excluded_callback,
                &mut query_excluded_info,
                physics_data,
            );
            if !result.collided {
                break;
            }
            *result_count += 1;
            query_excluded_info.query_exclude[query_excluded_info.query_exclude_size] =
                result.collider;
            query_excluded_info.query_exclude_size += 1;
            unsafe {
                (*results_out.add(array_idx)) = vector_to_godot(result.pixel_witness1);
                (*results_out.add(array_idx + 1)) = vector_to_godot(result.pixel_witness2);
            }
            array_idx += 2;
            cpt += 1;
        }
        array_idx > 0
    }

    pub unsafe fn rest_info(
        &mut self,
        shape_rid: Rid,
        transform: Transform,
        motion: Vector,
        _margin: f32,
        collision_mask: u32,
        collide_with_bodies: bool,
        collide_with_areas: bool,
        rest_info: *mut PhysicsServerExtensionShapeRestInfo,
        physics_data: &PhysicsData,
    ) -> bool {
        let Some(shape) = physics_data.shapes.get(&shape_rid) else {
            return false;
        };
        let Some(space) = physics_data.spaces.get(&self.space) else {
            return false;
        };
        if !space.is_valid() {
            return false;
        }
        let rapier_motion = vector_to_rapier(motion);
        let shape_info = shape_info_from_body_shape(shape.get_handle(), transform);
        let mut query_excluded_info = QueryExcludedInfo::default();
        query_excluded_info.query_collision_layer_mask = collision_mask;
        let result = shape_casting(
            space.get_handle(),
            rapier_motion,
            shape_info,
            collide_with_bodies,
            collide_with_areas,
            is_handle_excluded_callback,
            &mut query_excluded_info,
            physics_data,
        );
        if !result.collided {
            return false;
        }
        let (rid, shape_index) = RapierCollisionObject::get_collider_user_data(&result.user_data);
        let collision_object_2d = physics_data.collision_objects.get(&rid);
        let r_info = &mut *rest_info;
        if let Some(collision_object_2d) = collision_object_2d {
            let instance_id = collision_object_2d.get_base().get_instance_id();
            r_info.collider_id = ObjectId { id: instance_id };
            if let Some(body) = collision_object_2d.get_body() {
                let rel_vec = r_info.point
                    - (body.get_base().get_transform().origin + body.get_center_of_mass());
                r_info.linear_velocity = body.get_linear_velocity(&physics_data.physics_engine)
                    + cross_product(body.get_angular_velocity(&physics_data.physics_engine), rel_vec);
            } else {
                r_info.linear_velocity = Vector::default();
            }
            r_info.normal = vector_to_godot(result.normal1);
            r_info.rid = rid;
            r_info.shape = shape_index as i32;
        }
        true
    }
}
