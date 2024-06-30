use std::ffi::c_void;

#[cfg(feature = "dim2")]
use godot::engine::physics_server_2d::*;
#[cfg(feature = "dim3")]
use godot::engine::physics_server_3d::*;
use godot::engine::utilities::rid_allocate_id;
use godot::engine::utilities::rid_from_int64;
use godot::prelude::*;

use super::rapier_physics_server_extra::PhysicsData;
use super::rapier_project_settings::RapierProjectSettings;
use crate::bodies::rapier_area::RapierArea;
use crate::bodies::rapier_body::RapierBody;
use crate::bodies::rapier_collision_object::IRapierCollisionObject;
#[cfg(feature = "dim2")]
use crate::joints::rapier_damped_spring_joint_2d::RapierDampedSpringJoint2D;
#[cfg(feature = "dim2")]
use crate::joints::rapier_groove_joint_2d::RapierGrooveJoint2D;
use crate::joints::rapier_joint::IRapierJoint;
use crate::joints::rapier_joint::RapierEmptyJoint;
#[cfg(feature = "dim2")]
use crate::joints::rapier_pin_joint_2d::RapierPinJoint2D;
use crate::rapier_wrapper::prelude::*;
use crate::shapes::rapier_capsule_shape::RapierCapsuleShape;
use crate::shapes::rapier_circle_shape::RapierCircleShape;
#[cfg(feature = "dim2")]
use crate::shapes::rapier_concave_polygon_shape_2d::RapierConcavePolygonShape2D;
use crate::shapes::rapier_convex_polygon_shape::RapierConvexPolygonShape;
use crate::shapes::rapier_rectangle_shape::RapierRectangleShape;
#[cfg(feature = "dim2")]
use crate::shapes::rapier_segment_shape_2d::RapierSegmentShape2D;
use crate::shapes::rapier_separation_ray_shape::RapierSeparationRayShape;
use crate::shapes::rapier_shape::RapierShapeBase;
use crate::shapes::rapier_world_boundary_shape::RapierWorldBoundaryShape;
use crate::spaces::rapier_space::RapierSpace;
use crate::types::*;
pub struct RapierPhysicsServerImpl {
    active: bool,
    flushing_queries: bool,
    doing_sync: bool,
    island_count: i32,
    active_objects: i32,
    collision_pairs: i32,
    length_unit: real,
    max_ccd_substeps: usize,
    num_additional_friction_iterations: usize,
    num_internal_pgs_iterations: usize,
    num_solver_iterations: usize,
    pub physics_data: PhysicsData,
}
impl RapierPhysicsServerImpl {
    pub(super) fn default() -> Self {
        Self {
            active: true,
            flushing_queries: false,
            doing_sync: false,
            island_count: 0,
            active_objects: 0,
            collision_pairs: 0,
            length_unit: RapierProjectSettings::get_length_unit(),
            max_ccd_substeps: RapierProjectSettings::get_solver_max_ccd_substeps() as usize,
            num_additional_friction_iterations:
                RapierProjectSettings::get_solver_num_additional_friction_iterations() as usize,
            num_internal_pgs_iterations:
                RapierProjectSettings::get_solver_num_internal_pgs_iterations() as usize,
            num_solver_iterations: RapierProjectSettings::get_solver_num_solver_iterations()
                as usize,
            physics_data: PhysicsData::default(),
        }
    }

    pub(super) fn world_boundary_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierWorldBoundaryShape::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    pub(super) fn separation_ray_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierSeparationRayShape::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    #[cfg(feature = "dim2")]
    pub(super) fn segment_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierSegmentShape2D::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    pub(super) fn circle_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierCircleShape::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    pub(super) fn rectangle_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierRectangleShape::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    pub(super) fn capsule_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierCapsuleShape::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    pub(super) fn convex_polygon_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierConvexPolygonShape::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    #[cfg(feature = "dim2")]
    pub(super) fn concave_polygon_shape_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let shape = RapierConcavePolygonShape2D::new(rid);
        self.physics_data.shapes.insert(rid, Box::new(shape));
        rid
    }

    #[cfg(feature = "dim3")]
    pub(super) fn concave_polygon_shape_create(&mut self) -> Rid {
        Rid::Invalid
    }

    #[cfg(feature = "dim3")]
    pub(super) fn heightmap_shape_create(&mut self) -> Rid {
        Rid::Invalid
    }

    pub(super) fn shape_set_data(&mut self, shape: Rid, data: Variant) {
        let mut owners = None;
        if let Some(shape) = self.physics_data.shapes.get_mut(&shape) {
            shape.set_data(data, &mut self.physics_data.physics_engine);
            if shape.get_base().is_valid() {
                owners = Some(shape.get_base().get_owners().clone());
            }
        }
        if let Some(owners) = owners {
            RapierShapeBase::call_shape_changed(owners, shape, &mut self.physics_data);
        }
    }

    pub(super) fn shape_get_type(&self, shape: Rid) -> ShapeType {
        if let Some(shape) = self.physics_data.shapes.get(&shape) {
            return shape.get_type();
        }
        ShapeType::CUSTOM
    }

    pub(super) fn shape_get_data(&self, shape: Rid) -> Variant {
        if let Some(shape) = self.physics_data.shapes.get(&shape) {
            if shape.get_base().is_valid() {
                return shape.get_data();
            }
        }
        Variant::nil()
    }

    pub(super) fn shape_get_custom_solver_bias(&self, _shape: Rid) -> f32 {
        0.0
    }

    pub(super) unsafe fn shape_collide(
        &mut self,
        shape_a: Rid,
        xform_a: Transform,
        motion_a: Vector,
        shape_b: Rid,
        xform_b: Transform,
        motion_b: Vector,
        results: *mut c_void,
        result_max: i32,
        result_count: *mut i32,
    ) -> bool {
        let result = self.physics_data.shapes.get_many_mut([&shape_a, &shape_b]);
        if result.is_none() {
            return false;
        }
        let [shape_a, shape_b] = result.unwrap();
        let shape_a_handle = shape_a.get_base().get_handle();
        let shape_b_handle = shape_b.get_base().get_handle();
        if !shape_a.get_base().is_valid() || !shape_b.get_base().is_valid() {
            return false;
        }
        let shape_a_info = shape_info_from_body_shape(shape_a_handle, xform_a);
        let shape_b_info = shape_info_from_body_shape(shape_b_handle, xform_b);
        let rapier_a_motion = vector_to_rapier(motion_a);
        let rapier_b_motion = vector_to_rapier(motion_b);
        let results_out: *mut Vector = results as *mut Vector;
        let vector2_slice: &mut [Vector] =
            unsafe { std::slice::from_raw_parts_mut(results_out, result_max as usize) };
        let result = self.physics_data.physics_engine.shape_collide(
            rapier_a_motion,
            shape_a_info,
            rapier_b_motion,
            shape_b_info,
        );
        if !result.collided {
            return false;
        }
        *result_count = 1;
        vector2_slice[0] = vector_to_godot(result.pixel_witness1);
        vector2_slice[1] = vector_to_godot(result.pixel_witness2);
        true
    }

    pub(super) fn space_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let space = RapierSpace::new(rid, &mut self.physics_data.physics_engine);
        self.physics_data.spaces.insert(rid, space);
        rid
    }

    pub(super) fn space_set_active(&mut self, space_rid: Rid, active: bool) {
        if let Some(space) = self.physics_data.spaces.get(&space_rid) {
            if active {
                self.physics_data
                    .active_spaces
                    .insert(space.get_handle(), space_rid);
            } else {
                self.physics_data.active_spaces.remove(&space.get_handle());
            }
        }
    }

    pub(super) fn space_is_active(&self, space: Rid) -> bool {
        if let Some(space) = self.physics_data.spaces.get(&space) {
            return self
                .physics_data
                .active_spaces
                .contains_key(&space.get_handle());
        }
        false
    }

    pub(super) fn space_set_param(&mut self, _space: Rid, _param: SpaceParameter, _value: f32) {}

    pub(super) fn space_get_param(&self, _space: Rid, _param: SpaceParameter) -> f32 {
        0.0
    }

    pub(super) fn space_get_direct_state(
        &mut self,
        space: Rid,
    ) -> Option<Gd<PhysicsDirectSpaceState>> {
        if let Some(space) = self.physics_data.spaces.get(&space) {
            return space.get_direct_state().clone();
        }
        None
    }

    pub(super) fn space_set_debug_contacts(&mut self, space: Rid, max_contacts: i32) {
        if let Some(space) = self.physics_data.spaces.get_mut(&space) {
            space.set_debug_contacts(max_contacts);
        }
    }

    pub(super) fn space_get_contacts(&self, space: Rid) -> PackedVectorArray {
        if let Some(space) = self.physics_data.spaces.get(&space) {
            return space.get_debug_contacts().clone();
        }
        PackedVectorArray::new()
    }

    pub(super) fn space_get_contact_count(&self, space: Rid) -> i32 {
        if let Some(space) = self.physics_data.spaces.get(&space) {
            return space.get_debug_contact_count();
        }
        0
    }

    pub(super) fn area_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let area = RapierArea::new(rid);
        self.physics_data
            .collision_objects
            .insert(rid, Box::new(area));
        rid
    }

    pub(super) fn area_set_space(&mut self, area: Rid, space: Rid) {
        RapierArea::clear_detected_bodies(
            &area,
            &mut self.physics_data.spaces,
            &mut self.physics_data.collision_objects,
        );
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.set_space(
                space,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
        }
    }

    pub(super) fn area_get_space(&self, area: Rid) -> Rid {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_space();
        }
        Rid::Invalid
    }

    pub(super) fn area_add_shape(
        &mut self,
        area: Rid,
        shape: Rid,
        transform: Transform,
        disabled: bool,
    ) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.add_shape(
                shape,
                transform,
                disabled,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
        }
    }

    pub(super) fn area_set_shape(&mut self, area: Rid, shape_idx: i32, shape: Rid) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.set_shape(
                shape_idx as usize,
                shape,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
        }
    }

    pub(super) fn area_set_shape_transform(
        &mut self,
        area: Rid,
        shape_idx: i32,
        transform: Transform,
    ) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.set_shape_transform(
                shape_idx as usize,
                transform,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
        }
    }

    pub(super) fn area_set_shape_disabled(&mut self, area: Rid, shape_idx: i32, disabled: bool) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.set_shape_disabled(
                shape_idx as usize,
                disabled,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
        }
    }

    pub(super) fn area_get_shape_count(&self, area: Rid) -> i32 {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_shape_count();
        }
        -1
    }

    pub(super) fn area_get_shape(&self, area: Rid, shape_idx: i32) -> Rid {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_shape(shape_idx as usize);
        }
        Rid::Invalid
    }

    pub(super) fn area_get_shape_transform(&self, area: Rid, shape_idx: i32) -> Transform {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_shape_transform(shape_idx as usize);
        }
        Transform::default()
    }

    pub(super) fn area_remove_shape(&mut self, area: Rid, shape_idx: i32) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.remove_shape_idx(
                shape_idx as usize,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
        }
    }

    pub(super) fn area_clear_shapes(&mut self, area: Rid) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            while area.get_base().get_shape_count() > 0 {
                area.remove_shape_idx(
                    0,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
        }
    }

    pub(super) fn area_attach_object_instance_id(&mut self, area: Rid, id: u64) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.get_mut_base().set_instance_id(id);
        }
    }

    pub(super) fn area_get_object_instance_id(&self, area: Rid) -> u64 {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_instance_id();
        }
        0
    }

    pub(super) fn area_attach_canvas_instance_id(&mut self, area: Rid, id: u64) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.get_mut_base().set_canvas_instance_id(id);
        }
    }

    pub(super) fn area_get_canvas_instance_id(&self, area: Rid) -> u64 {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_canvas_instance_id();
        }
        0
    }

    pub(super) fn area_set_param(&mut self, area: Rid, param: AreaParameter, value: Variant) {
        if let Some(space) = self.physics_data.spaces.get_mut(&area) {
            space.set_default_area_param(param, value);
            return;
        }
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            if let Some(area) = area.get_mut_area() {
                area.set_param(param, value, &mut self.physics_data.spaces);
            }
        }
    }

    pub(super) fn area_set_transform(&mut self, area: Rid, transform: Transform) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.get_mut_base().set_transform(
                transform,
                false,
                &mut self.physics_data.physics_engine,
            );
        }
    }

    pub(super) fn area_get_param(&self, area: Rid, param: AreaParameter) -> Variant {
        if let Some(space) = self.physics_data.spaces.get(&area) {
            return space.get_default_area_param(param);
        }
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            if let Some(area) = area.get_area() {
                return area.get_param(param);
            }
        }
        Variant::nil()
    }

    pub(super) fn area_get_transform(&self, area: Rid) -> Transform {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_transform();
        }
        Transform::default()
    }

    pub(super) fn area_set_collision_layer(&mut self, area: Rid, layer: u32) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.get_mut_base().set_collision_layer(layer);
        }
    }

    pub(super) fn area_get_collision_layer(&self, area: Rid) -> u32 {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_collision_layer();
        }
        0
    }

    pub(super) fn area_set_collision_mask(&mut self, area: Rid, mask: u32) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.get_mut_base().set_collision_mask(mask);
        }
    }

    pub(super) fn area_get_collision_mask(&self, area: Rid) -> u32 {
        if let Some(area) = self.physics_data.collision_objects.get(&area) {
            return area.get_base().get_collision_mask();
        }
        0
    }

    pub(super) fn area_set_monitorable(&mut self, area: Rid, monitorable: bool) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            if let Some(area) = area.get_mut_area() {
                area.set_monitorable(monitorable);
            }
        }
    }

    pub(super) fn area_set_pickable(&mut self, area: Rid, pickable: bool) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.get_mut_base().set_pickable(pickable);
        }
    }

    #[cfg(feature = "dim3")]
    pub(super) fn area_set_ray_pickable(&mut self, area: Rid, pickable: bool) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            area.get_mut_base().set_pickable(pickable);
        }
    }

    pub(super) fn area_set_monitor_callback(&mut self, area: Rid, callback: Callable) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            if let Some(area) = area.get_mut_area() {
                area.set_monitor_callback(callback);
            }
        }
    }

    pub(super) fn area_set_area_monitor_callback(&mut self, area: Rid, callback: Callable) {
        if let Some(area) = self.physics_data.collision_objects.get_mut(&area) {
            if let Some(area) = area.get_mut_area() {
                area.set_area_monitor_callback(callback);
            }
        }
    }

    pub(super) fn body_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        let body = RapierBody::new(rid);
        self.physics_data
            .collision_objects
            .insert(rid, Box::new(body));
        rid
    }

    pub(super) fn body_set_space(&mut self, body: Rid, space: Rid) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.set_space(
                space,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
        }
    }

    pub(super) fn body_get_space(&self, body: Rid) -> Rid {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            return body.get_base().get_space();
        }
        Rid::Invalid
    }

    pub(super) fn body_set_mode(&mut self, body: Rid, mode: BodyMode) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body)
            && let Some(body) = body.get_mut_body()
        {
            body.set_mode(
                mode,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
            );
        }
        let mut area_override_settings = None;
        if let Some(body) = self.physics_data.collision_objects.get(&body)
            && let Some(body) = body.get_body()
        {
            area_override_settings = Some(body.get_area_override_settings(
                &mut self.physics_data.spaces,
                &self.physics_data.collision_objects,
            ));
        }
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body)
            && let Some(body) = body.get_mut_body()
        {
            if let Some(area_override_settings) = area_override_settings {
                body.apply_area_override(
                    area_override_settings,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                );
            }
        }
    }

    pub(super) fn body_get_mode(&self, body: Rid) -> BodyMode {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_base().get_mode();
            }
        }
        BodyMode::STATIC
    }

    pub(super) fn body_add_shape(
        &mut self,
        body: Rid,
        shape: Rid,
        transform: Transform,
        disabled: bool,
    ) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.add_shape(
                    shape,
                    transform,
                    disabled,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
        }
    }

    pub(super) fn body_set_shape(&mut self, body: Rid, shape_idx: i32, shape: Rid) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_shape(
                    shape_idx as usize,
                    shape,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
        }
    }

    pub(super) fn body_set_shape_transform(
        &mut self,
        body: Rid,
        shape_idx: i32,
        transform: Transform,
    ) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_shape_transform(
                    shape_idx as usize,
                    transform,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
        }
    }

    pub(super) fn body_get_shape_count(&self, body: Rid) -> i32 {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_base().get_shape_count();
            }
        }
        0
    }

    pub(super) fn body_get_shape(&self, body: Rid, shape_idx: i32) -> Rid {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_base().get_shape(shape_idx as usize);
            }
        }
        Rid::Invalid
    }

    pub(super) fn body_get_shape_transform(&self, body: Rid, shape_idx: i32) -> Transform {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_base().get_shape_transform(shape_idx as usize);
            }
        }
        Transform::default()
    }

    pub(super) fn body_set_shape_disabled(&mut self, body: Rid, shape_idx: i32, disabled: bool) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_shape_disabled(
                    shape_idx as usize,
                    disabled,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
        }
    }

    pub(super) fn body_set_shape_as_one_way_collision(
        &mut self,
        body: Rid,
        shape_idx: i32,
        enable: bool,
        margin: f32,
    ) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base()
                .set_shape_as_one_way_collision(shape_idx as usize, enable, margin);
        }
    }

    pub(super) fn body_remove_shape(&mut self, body: Rid, shape_idx: i32) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.remove_shape_idx(
                    shape_idx as usize,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
        }
    }

    pub(super) fn body_clear_shapes(&mut self, body: Rid) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            while body.get_base().get_shape_count() > 0 {
                body.remove_shape_idx(
                    0,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
        }
    }

    pub(super) fn body_attach_object_instance_id(&mut self, body: Rid, id: u64) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base().set_instance_id(id);
        }
    }

    pub(super) fn body_get_object_instance_id(&self, body: Rid) -> u64 {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            return body.get_base().get_instance_id();
        }
        0
    }

    pub(super) fn body_attach_canvas_instance_id(&mut self, body: Rid, id: u64) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base().set_canvas_instance_id(id);
        }
    }

    pub(super) fn body_get_canvas_instance_id(&self, body: Rid) -> u64 {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            return body.get_base().get_canvas_instance_id();
        }
        0
    }

    #[cfg(feature = "dim2")]
    pub(super) fn body_set_continuous_collision_detection_mode(
        &mut self,
        body: Rid,
        mode: CcdMode,
    ) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_continuous_collision_detection_mode(
                    mode != CcdMode::DISABLED,
                    &mut self.physics_data.physics_engine,
                );
            }
        }
    }

    #[cfg(feature = "dim2")]
    pub(super) fn body_get_continuous_collision_detection_mode(&self, body: Rid) -> CcdMode {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                if body.get_continuous_collision_detection_mode() {
                    return CcdMode::CAST_RAY;
                }
            }
        }
        CcdMode::DISABLED
    }

    #[cfg(feature = "dim3")]
    pub fn body_set_enable_continuous_collision_detection(&mut self, body: Rid, enable: bool) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_continuous_collision_detection_mode(
                    enable,
                    &mut self.physics_data.physics_engine,
                );
            }
        }
    }

    #[cfg(feature = "dim3")]
    pub(super) fn body_is_continuous_collision_detection_enabled(&self, body: Rid) -> bool {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                if body.get_continuous_collision_detection_mode() {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn body_set_collision_layer(&mut self, body: Rid, layer: u32) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base().set_collision_layer(layer);
        }
    }

    pub(super) fn body_get_collision_layer(&self, body: Rid) -> u32 {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            return body.get_base().get_collision_layer();
        }
        0
    }

    pub(super) fn body_set_collision_mask(&mut self, body: Rid, mask: u32) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base().set_collision_mask(mask);
        }
    }

    pub(super) fn body_get_collision_mask(&self, body: Rid) -> u32 {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            return body.get_base().get_collision_mask();
        }
        0
    }

    pub(super) fn body_set_collision_priority(&mut self, body: Rid, priority: f32) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base().set_collision_priority(priority);
        }
    }

    pub(super) fn body_get_collision_priority(&self, body: Rid) -> f32 {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            return body.get_base().get_collision_priority();
        }
        0.0
    }

    pub(super) fn body_set_param(&mut self, body: Rid, param: BodyParameter, value: Variant) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_param(
                    param,
                    value,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                );
            }
        }
    }

    pub(super) fn body_get_param(&self, body: Rid, param: BodyParameter) -> Variant {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_param(param);
            }
        }
        Variant::nil()
    }

    pub(super) fn body_reset_mass_properties(&mut self, body: Rid) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.reset_mass_properties(
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                );
            }
        }
    }

    pub(super) fn body_set_state(&mut self, body: Rid, state: BodyState, value: Variant) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_state(
                    state,
                    value,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                );
            }
        }
    }

    pub(super) fn body_get_state(&self, body: Rid, state: BodyState) -> Variant {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_state(state, &self.physics_data.physics_engine);
            }
        }
        Variant::nil()
    }

    pub(super) fn body_apply_central_impulse(&mut self, body: Rid, impulse: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.apply_central_impulse(impulse, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_apply_torque_impulse(&mut self, body: Rid, impulse: Angle) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.apply_torque_impulse(impulse, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_apply_impulse(&mut self, body: Rid, impulse: Vector, position: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.apply_impulse(impulse, position, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_apply_central_force(&mut self, body: Rid, force: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.apply_central_force(force, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_apply_force(&mut self, body: Rid, force: Vector, position: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.apply_force(force, position, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_apply_torque(&mut self, body: Rid, torque: Angle) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.apply_torque(torque, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_add_constant_central_force(&mut self, body: Rid, force: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.add_constant_central_force(force, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_add_constant_force(&mut self, body: Rid, force: Vector, position: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.add_constant_force(force, position, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_add_constant_torque(&mut self, body: Rid, torque: Angle) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.add_constant_torque(torque, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_set_constant_force(&mut self, body: Rid, force: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.set_constant_force(force, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_get_constant_force(&self, body: Rid) -> Vector {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_constant_force(&self.physics_data.physics_engine);
            }
        }
        Vector::default()
    }

    pub(super) fn body_set_constant_torque(&mut self, body: Rid, torque: Angle) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.force_mass_update(
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                    &mut self.physics_data.physics_engine,
                );
                body.set_constant_torque(torque, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_get_constant_torque(&self, body: Rid) -> Angle {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_constant_torque(&self.physics_data.physics_engine);
            }
        }
        ANGLE_ZERO
    }

    pub(super) fn body_set_axis_velocity(&mut self, body: Rid, axis_velocity: Vector) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                let axis_velocity = axis_velocity;
                let mut v = body.get_linear_velocity(&mut self.physics_data.physics_engine);
                let axis = vector_normalized(axis_velocity);
                v -= axis * axis.dot(v);
                v += axis_velocity;
                body.set_linear_velocity(v, &mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_add_collision_exception(&mut self, body: Rid, excepted_body: Rid) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.add_exception(excepted_body);
                body.wakeup(&mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_remove_collision_exception(&mut self, body: Rid, excepted_body: Rid) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.remove_exception(excepted_body);
                body.wakeup(&mut self.physics_data.physics_engine);
            }
        }
    }

    pub(super) fn body_get_collision_exceptions(&self, body: Rid) -> Array<Rid> {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                let exceptions = body.get_exceptions();
                let mut arr = Array::new();
                for e in exceptions {
                    arr.push(*e);
                }
                return arr;
            }
        }
        Array::new()
    }

    pub(super) fn body_set_max_contacts_reported(&mut self, body: Rid, amount: i32) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_max_contacts_reported(amount);
            }
        }
    }

    pub(super) fn body_get_max_contacts_reported(&self, body: Rid) -> i32 {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_max_contacts_reported();
            }
        }
        0
    }

    pub(super) fn body_set_contacts_reported_depth_threshold(
        &mut self,
        _body: Rid,
        _threshold: f32,
    ) {
    }

    pub(super) fn body_get_contacts_reported_depth_threshold(&self, _body: Rid) -> f32 {
        0.0
    }

    pub(super) fn body_set_omit_force_integration(&mut self, body: Rid, enable: bool) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_omit_force_integration(enable);
            }
        }
    }

    pub(super) fn body_is_omitting_force_integration(&self, body: Rid) -> bool {
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                return body.get_omit_force_integration();
            }
        }
        false
    }

    pub(super) fn body_set_state_sync_callback(&mut self, body: Rid, callable: Callable) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_state_sync_callback(callable);
            }
        }
    }

    pub(super) fn body_set_force_integration_callback(
        &mut self,
        body: Rid,
        callable: Callable,
        userdata: Variant,
    ) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                body.set_force_integration_callback(callable, userdata);
            }
        }
    }

    pub(super) unsafe fn body_collide_shape(
        &mut self,
        body: Rid,
        body_shape: i32,
        shape: Rid,
        shape_xform: Transform,
        motion: Vector,
        results: *mut c_void,
        result_max: i32,
        result_count: *mut i32,
    ) -> bool {
        let mut body_shape_rid = Rid::Invalid;
        let mut body_transform = Transform::IDENTITY;
        let mut body_shape_transform = Transform::IDENTITY;
        {
            if let Some(body) = self.physics_data.collision_objects.get(&body) {
                if let Some(body) = body.get_body() {
                    body_shape_rid = body.get_base().get_shape(body_shape as usize);
                    body_transform = body.get_base().get_transform();
                    body_shape_transform = body.get_base().get_shape_transform(body_shape as usize);
                }
            }
        }
        self.shape_collide(
            body_shape_rid,
            body_transform * body_shape_transform,
            Vector::ZERO,
            shape,
            shape_xform,
            motion,
            results,
            result_max,
            result_count,
        )
    }

    pub(super) fn body_set_pickable(&mut self, body: Rid, pickable: bool) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base().set_pickable(pickable);
        }
    }

    #[cfg(feature = "dim3")]
    pub(super) fn body_set_ray_pickable(&mut self, body: Rid, pickable: bool) {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            body.get_mut_base().set_pickable(pickable);
        }
    }

    pub(super) fn body_get_direct_state(
        &mut self,
        body: Rid,
    ) -> Option<Gd<PhysicsDirectBodyState>> {
        if let Some(body) = self.physics_data.collision_objects.get_mut(&body) {
            if let Some(body) = body.get_mut_body() {
                return body.get_direct_state().cloned();
            }
        }
        None
    }

    pub unsafe fn body_test_motion(
        &self,
        body: Rid,
        from: Transform,
        motion: Vector,
        margin: f32,
        collide_separation_ray: bool,
        recovery_as_collision: bool,
        result: *mut PhysicsServerExtensionMotionResult,
    ) -> bool {
        // need to get mutable reference
        if let Some(body) = self.physics_data.collision_objects.get(&body) {
            if let Some(body) = body.get_body() {
                if let Some(space) = self.physics_data.spaces.get(&body.get_base().get_space()) {
                    let result: &mut PhysicsServerExtensionMotionResult = &mut *result;
                    return space.test_body_motion(
                        body,
                        from,
                        motion,
                        margin,
                        collide_separation_ray,
                        recovery_as_collision,
                        result,
                        &self.physics_data.physics_engine,
                        &self.physics_data.shapes,
                        &self.physics_data.collision_objects,
                    );
                }
            }
        }
        false
    }

    pub(super) fn joint_create(&mut self) -> Rid {
        let rid = rid_from_int64(rid_allocate_id());
        self.physics_data
            .joints
            .insert(rid, Box::new(RapierEmptyJoint::new()));
        rid
    }

    pub(super) fn joint_clear(&mut self, rid: Rid) {
        if let Some(mut prev_joint) = self.physics_data.joints.remove(&rid) {
            let mut joint = RapierEmptyJoint::new();
            joint
                .get_mut_base()
                .copy_settings_from(prev_joint.get_base(), &mut self.physics_data.physics_engine);
            prev_joint
                .get_mut_base()
                .destroy_joint(&mut self.physics_data.physics_engine);
            self.physics_data.joints.insert(rid, Box::new(joint));
        }
    }

    #[cfg(feature = "dim2")]
    pub(super) fn joint_set_param(&mut self, joint: Rid, param: JointParam, value: f32) {
        if let Some(joint) = self.physics_data.joints.get_mut(&joint) {
            if param == JointParam::MAX_FORCE {
                joint.get_mut_base().set_max_force(value);
            }
        }
    }

    #[cfg(feature = "dim2")]
    pub(super) fn joint_get_param(&self, joint: Rid, param: JointParam) -> f32 {
        if let Some(joint) = self.physics_data.joints.get(&joint) {
            if param == JointParam::MAX_FORCE {
                return joint.get_base().get_max_force();
            }
        }
        0.0
    }

    pub(super) fn joint_disable_collisions_between_bodies(&mut self, joint: Rid, disable: bool) {
        if let Some(joint) = self.physics_data.joints.get_mut(&joint) {
            joint
                .get_mut_base()
                .disable_collisions_between_bodies(disable, &mut self.physics_data.physics_engine);
        }
    }

    pub(super) fn joint_is_disabled_collisions_between_bodies(&self, joint: Rid) -> bool {
        if let Some(joint) = self.physics_data.joints.get(&joint) {
            return joint.get_base().is_disabled_collisions_between_bodies();
        }
        true
    }

    #[cfg(feature = "dim3")]
    pub(super) fn joint_make_pin(
        &mut self,
        joint: Rid,
        body_a: Rid,
        local_a: Vector3,
        body_b: Rid,
        local_b: Vector3,
    ) {
        return;
    }

    #[cfg(feature = "dim3")]
    pub(super) fn pin_joint_set_param(&mut self, joint: Rid, param: PinJointParam, value: f32) {}

    #[cfg(feature = "dim3")]
    pub(super) fn pin_joint_get_param(&self, joint: Rid, param: PinJointParam) -> f32 {
        return 0.0;
    }

    #[cfg(feature = "dim3")]
    pub(super) fn pin_joint_set_local_a(&mut self, joint: Rid, local_A: Vector3) {
        return;
    }

    #[cfg(feature = "dim3")]
    pub(super) fn pin_joint_get_local_a(&self, joint: Rid) -> Vector3 {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    #[cfg(feature = "dim3")]
    pub(super) fn pin_joint_set_local_b(&mut self, joint: Rid, local_B: Vector3) {
        return;
    }

    #[cfg(feature = "dim3")]
    pub(super) fn pin_joint_get_local_b(&self, joint: Rid) -> Vector3 {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    #[cfg(feature = "dim3")]
    pub(super) fn joint_make_hinge(
        &mut self,
        joint: Rid,
        body_A: Rid,
        hinge_A: Transform3D,
        body_B: Rid,
        hinge_B: Transform3D,
    ) {
        unimplemented!()
    }

    #[cfg(feature = "dim3")]
    pub(super) fn joint_make_hinge_simple(
        &mut self,
        joint: Rid,
        body_A: Rid,
        pivot_A: Vector3,
        axis_A: Vector3,
        body_B: Rid,
        pivot_B: Vector3,
        axis_B: Vector3,
    ) {
        unimplemented!()
    }

    #[cfg(feature = "dim3")]
    pub(super) fn hinge_joint_set_param(&mut self, joint: Rid, param: HingeJointParam, value: f32) {
        unimplemented!()
    }

    #[cfg(feature = "dim3")]
    pub(super) fn hinge_joint_get_param(&self, joint: Rid, param: HingeJointParam) -> f32 {
        unimplemented!()
    }

    #[cfg(feature = "dim3")]
    pub(super) fn hinge_joint_set_flag(&mut self, joint: Rid, flag: HingeJointFlag, enabled: bool) {
        unimplemented!()
    }

    #[cfg(feature = "dim3")]
    pub(super) fn hinge_joint_get_flag(&self, joint: Rid, flag: HingeJointFlag) -> bool {
        unimplemented!()
    }

    #[cfg(feature = "dim2")]
    pub(super) fn joint_make_pin(&mut self, rid: Rid, anchor: Vector, body_a: Rid, body_b: Rid) {
        let mut joint: Box<dyn IRapierJoint>;
        if let Some(body_a) = self.physics_data.collision_objects.get(&body_a)
            && let Some(body_b) = self.physics_data.collision_objects.get(&body_b)
        {
            joint = Box::new(RapierPinJoint2D::new(
                anchor,
                body_a,
                body_b,
                &mut self.physics_data.physics_engine,
            ));
            if let Some(prev_joint) = self.physics_data.joints.remove(&rid) {
                joint.get_mut_base().copy_settings_from(
                    prev_joint.get_base(),
                    &mut self.physics_data.physics_engine,
                );
            }
        } else {
            joint = Box::new(RapierEmptyJoint::new());
        }
        self.physics_data.joints.insert(rid, joint);
    }

    #[cfg(feature = "dim2")]
    pub(super) fn joint_make_groove(
        &mut self,
        rid: Rid,
        a_groove1: Vector,
        a_groove2: Vector,
        b_anchor: Vector,
        body_a: Rid,
        body_b: Rid,
    ) {
        let mut joint: Box<dyn IRapierJoint>;
        if let Some(body_a) = self.physics_data.collision_objects.get(&body_a)
            && let Some(body_b) = self.physics_data.collision_objects.get(&body_b)
        {
            joint = Box::new(RapierGrooveJoint2D::new(
                a_groove1,
                a_groove2,
                b_anchor,
                body_a,
                body_b,
                &mut self.physics_data.physics_engine,
            ));
            if let Some(prev_joint) = self.physics_data.joints.remove(&rid) {
                joint.get_mut_base().copy_settings_from(
                    prev_joint.get_base(),
                    &mut self.physics_data.physics_engine,
                );
            }
        } else {
            joint = Box::new(RapierEmptyJoint::new());
        }
        self.physics_data.joints.insert(rid, joint);
    }

    #[cfg(feature = "dim2")]
    pub(super) fn joint_make_damped_spring(
        &mut self,
        rid: Rid,
        anchor_a: Vector,
        anchor_b: Vector,
        body_a: Rid,
        body_b: Rid,
    ) {
        let mut joint: Box<dyn IRapierJoint>;
        if let Some(body_a) = self.physics_data.collision_objects.get(&body_a)
            && let Some(body_b) = self.physics_data.collision_objects.get(&body_b)
        {
            joint = Box::new(RapierDampedSpringJoint2D::new(
                anchor_a,
                anchor_b,
                body_a,
                body_b,
                &mut self.physics_data.physics_engine,
            ));
            if let Some(prev_joint) = self.physics_data.joints.remove(&rid) {
                joint.get_mut_base().copy_settings_from(
                    prev_joint.get_base(),
                    &mut self.physics_data.physics_engine,
                );
            }
        } else {
            joint = Box::new(RapierEmptyJoint::new());
        }
        self.physics_data.joints.insert(rid, joint);
    }

    #[cfg(feature = "dim2")]
    pub(super) fn pin_joint_set_flag(&mut self, joint: Rid, flag: PinJointFlag, enabled: bool) {
        if let Some(joint) = self.physics_data.joints.get_mut(&joint) {
            if let Some(joint) = joint.get_mut_pin() {
                joint.set_flag(flag, enabled, &mut self.physics_data.physics_engine);
            }
        }
    }

    #[cfg(feature = "dim2")]
    pub(super) fn pin_joint_get_flag(&self, joint: Rid, flag: PinJointFlag) -> bool {
        if let Some(joint) = self.physics_data.joints.get(&joint) {
            if let Some(joint) = joint.get_pin() {
                return joint.get_flag(flag);
            }
        }
        false
    }

    #[cfg(feature = "dim2")]
    pub(super) fn pin_joint_set_param(&mut self, joint: Rid, param: PinJointParam, value: f32) {
        if let Some(joint) = self.physics_data.joints.get_mut(&joint) {
            if let Some(joint) = joint.get_mut_pin() {
                joint.set_param(param, value, &mut self.physics_data.physics_engine);
            }
        }
    }

    #[cfg(feature = "dim2")]
    pub(super) fn pin_joint_get_param(&self, joint: Rid, param: PinJointParam) -> f32 {
        if let Some(joint) = self.physics_data.joints.get(&joint) {
            if let Some(joint) = joint.get_pin() {
                return joint.get_param(param);
            }
        }
        0.0
    }

    #[cfg(feature = "dim2")]
    pub(super) fn damped_spring_joint_set_param(
        &mut self,
        joint: Rid,
        param: DampedSpringParam,
        value: f32,
    ) {
        if let Some(joint) = self.physics_data.joints.get_mut(&joint) {
            if let Some(joint) = joint.get_mut_damped_spring() {
                joint.set_param(param, value, &mut self.physics_data.physics_engine);
            }
        }
    }

    #[cfg(feature = "dim2")]
    pub(super) fn damped_spring_joint_get_param(
        &self,
        joint: Rid,
        param: DampedSpringParam,
    ) -> f32 {
        if let Some(joint) = self.physics_data.joints.get(&joint) {
            if let Some(joint) = joint.get_damped_spring() {
                return joint.get_param(param);
            }
        }
        0.0
    }

    pub(super) fn joint_get_type(&self, joint: Rid) -> JointType {
        if let Some(joint) = self.physics_data.joints.get(&joint) {
            return joint.get_type();
        }
        JointType::MAX
    }

    pub(super) fn free_rid(&mut self, rid: Rid) {
        if let Some(mut shape) = self.physics_data.shapes.remove(&rid) {
            for (owner, _) in shape.get_base().get_owners() {
                if let Some(body) = self.physics_data.collision_objects.get_mut(owner) {
                    body.remove_shape_rid(
                        shape.get_base().get_rid(),
                        &mut self.physics_data.physics_engine,
                        &mut self.physics_data.spaces,
                        &mut self.physics_data.shapes,
                    );
                }
            }
            shape
                .get_mut_base()
                .destroy_shape(&mut self.physics_data.physics_engine);
            return;
        }
        if let Some(mut body) = self.physics_data.collision_objects.remove(&rid) {
            body.set_space(
                Rid::Invalid,
                &mut self.physics_data.physics_engine,
                &mut self.physics_data.spaces,
                &mut self.physics_data.shapes,
            );
            while body.get_base().get_shape_count() > 0 {
                body.remove_shape_idx(
                    0,
                    &mut self.physics_data.physics_engine,
                    &mut self.physics_data.spaces,
                    &mut self.physics_data.shapes,
                );
            }
            body.get_mut_base()
                .destroy_body(&mut self.physics_data.physics_engine);
            return;
        }
        if let Some(mut space) = self.physics_data.spaces.remove(&rid) {
            let space_handle = space.get_handle();
            space.destroy_space(&mut self.physics_data.physics_engine);
            if self
                .physics_data
                .active_spaces
                .remove(&space_handle)
                .is_some()
            {
                return;
            }
        }
        if let Some(mut joint) = self.physics_data.joints.remove(&rid) {
            joint
                .get_mut_base()
                .destroy_joint(&mut self.physics_data.physics_engine);
            return;
        }
        if let Some(mut fluid) = self.physics_data.fluids.remove(&rid) {
            fluid.destroy_fluid(&mut self.physics_data.physics_engine);
        }
    }

    pub(super) fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub(super) fn init_ext(&mut self) {
        self.active = true;
        self.flushing_queries = false;
        self.doing_sync = false;
        self.island_count = 0;
        self.active_objects = 0;
        self.collision_pairs = 0;
    }

    pub(super) fn step(&mut self, step: f32) {
        if !self.active {
            return;
        }
        self.island_count = 0;
        self.active_objects = 0;
        self.collision_pairs = 0;
        let active_spaces = self.physics_data.active_spaces.clone();
        for space_rid in active_spaces.values() {
            let settings = SimulationSettings {
                dt: step,
                length_unit: self.length_unit,
                max_ccd_substeps: self.max_ccd_substeps,
                num_additional_friction_iterations: self.num_additional_friction_iterations,
                num_internal_pgs_iterations: self.num_internal_pgs_iterations,
                num_solver_iterations: self.num_solver_iterations,
                pixel_gravity: vector_to_rapier(Vector::ZERO),
                pixel_liquid_gravity: vector_to_rapier(Vector::ZERO),
            };
            RapierSpace::step(step, space_rid, &mut self.physics_data, settings);
            if let Some(space) = self.physics_data.spaces.get(space_rid) {
                self.island_count += space.get_island_count();
                self.active_objects += space.get_active_objects();
                self.collision_pairs += space.get_collision_pairs();
            }
        }
    }

    pub(super) fn sync(&mut self) {
        self.doing_sync = true;
    }

    pub(super) fn flush_queries(&mut self) -> Vec<Callable> {
        if !self.active {
            return Vec::default();
        }
        self.flushing_queries = true;
        let active_spaces = self.physics_data.active_spaces.clone();
        let mut queries = Vec::default();
        for space in active_spaces.values() {
            if let Some(space) = self.physics_data.spaces.get_mut(space) {
                queries.append(&mut space.get_queries(&mut self.physics_data.collision_objects));
            }
        }
        queries
    }

    pub(super) fn end_sync(&mut self) {
        self.doing_sync = false;
    }

    pub(super) fn finish(&mut self) {}

    pub(super) fn is_flushing_queries(&self) -> bool {
        self.flushing_queries
    }

    pub(super) fn finish_flushing_queries(&mut self) {
        self.flushing_queries = false;
    }

    pub(super) fn get_process_info(&mut self, process_info: ProcessInfo) -> i32 {
        match process_info {
            ProcessInfo::ACTIVE_OBJECTS => self.active_objects,
            ProcessInfo::COLLISION_PAIRS => self.collision_pairs,
            ProcessInfo::ISLAND_COUNT => self.island_count,
            _ => 0,
        }
    }
}
impl Drop for RapierPhysicsServerImpl {
    fn drop(&mut self) {
        godot_error!("RapierPhysicsServer dropped");
    }
}
