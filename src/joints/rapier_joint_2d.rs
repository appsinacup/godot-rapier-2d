use crate::rapier2d::{
    handle::Handle,
    joint::{joint_change_disable_collision, joint_destroy},
};
use godot::{builtin::Rid, engine::physics_server_2d};

pub trait IRapierJoint2D {
    fn get_base(&self) -> &RapierJointBase2D;
    fn get_type(&self) -> physics_server_2d::JointType;
}

pub struct RapierJointBase2D {
    bias: f32,
    max_bias: f32,
    max_force: f32,
    rid: Rid,
    handle: Handle,
    space_handle: Handle,
    disabled_collisions_between_bodies: bool,
}

impl RapierJointBase2D {
    pub fn new(space_handle: Handle, handle: Handle, rid: Rid) -> Self {
        Self {
            bias: 0.0,
            max_bias: 3.40282e38,
            max_force: 3.40282e38,
            rid: rid,
            handle: handle,
            space_handle: space_handle,
            disabled_collisions_between_bodies: true,
        }
    }

    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    pub fn get_space_handle(&self) -> Handle {
        self.space_handle
    }

    pub fn set_handle(&mut self, handle: Handle) {
        self.handle = handle
    }

    pub fn set_max_force(&mut self, force: f32) {
        self.max_force = force;
    }

    pub fn get_max_force(&self) -> f32 {
        self.max_force
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias;
    }

    pub fn get_bias(&self) -> f32 {
        self.bias
    }

    pub fn set_max_bias(&mut self, bias: f32) {
        self.max_bias = bias;
    }

    pub fn get_max_bias(&self) -> f32 {
        self.max_bias
    }

    pub fn get_rid(&self) -> Rid {
        self.rid
    }

    // Careful when doing this you must also update the place where it's stored.
    pub fn set_rid(&mut self, rid: Rid) {
        self.rid = rid;
    }

    pub fn disable_collisions_between_bodies(&mut self, disabled: bool) {
        self.disabled_collisions_between_bodies = disabled;
        if self.handle.is_valid() {
            joint_change_disable_collision(
                self.space_handle,
                self.handle,
                self.disabled_collisions_between_bodies,
            );
        }
    }

    pub fn is_disabled_collisions_between_bodies(&self) -> bool {
        self.disabled_collisions_between_bodies
    }

    pub fn copy_settings_from(&mut self, joint: RapierJointBase2D) {
        self.set_rid(joint.get_rid());
        self.set_max_force(joint.get_max_force());
        self.set_bias(joint.get_bias());
        self.set_max_bias(joint.get_max_bias());
        self.disable_collisions_between_bodies(joint.is_disabled_collisions_between_bodies());
    }
}

impl Drop for RapierJointBase2D {
    fn drop(&mut self) {
        if self.handle.is_valid() {
            joint_destroy(self.space_handle, self.handle);
        }
    }
}
