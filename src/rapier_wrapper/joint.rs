use rapier::prelude::*;

use crate::rapier_wrapper::prelude::*;
impl PhysicsEngine {
    #[cfg(feature = "dim2")]
    pub fn joint_create_revolute(
        &mut self,
        world_handle: WorldHandle,
        body_handle_1: RigidBodyHandle,
        body_handle_2: RigidBodyHandle,
        anchor_1: Vector<Real>,
        anchor_2: Vector<Real>,
        angular_limit_lower: Real,
        angular_limit_upper: Real,
        angular_limit_enabled: bool,
        motor_target_velocity: Real,
        motor_enabled: bool,
        disable_collision: bool,
    ) -> ImpulseJointHandle {
        if let Some(physics_world) = self.get_mut_world(world_handle) {
            let mut joint = RevoluteJointBuilder::new()
                .local_anchor1(Point { coords: anchor_1 })
                .local_anchor2(Point { coords: anchor_2 })
                .motor_model(MotorModel::ForceBased)
                .contacts_enabled(!disable_collision);
            if angular_limit_enabled {
                joint = joint.limits([angular_limit_lower, angular_limit_upper]);
            }
            if motor_enabled {
                joint = joint.motor_velocity(motor_target_velocity, 0.0);
            }
            return physics_world.insert_joint(body_handle_1, body_handle_2, joint);
        }
        ImpulseJointHandle::invalid()
    }
    #[cfg(feature = "dim3")]
    pub fn joint_create_revolute(
        &mut self,
        world_handle: WorldHandle,
        body_handle_1: RigidBodyHandle,
        body_handle_2: RigidBodyHandle,
        anchor_1: Vector<Real>,
        anchor_2: Vector<Real>,
        angular_limit_lower: Real,
        angular_limit_upper: Real,
        angular_limit_enabled: bool,
        motor_target_velocity: Real,
        motor_enabled: bool,
        disable_collision: bool,
    ) -> ImpulseJointHandle {
        if let Some(physics_world) = self.get_mut_world(world_handle) {
            let mut joint = RevoluteJointBuilder::new()
                .local_anchor1(Point { coords: anchor_1 })
                .local_anchor2(Point { coords: anchor_2 })
                .motor_model(MotorModel::ForceBased)
                .contacts_enabled(!disable_collision);
            if angular_limit_enabled {
                joint = joint.limits([angular_limit_lower, angular_limit_upper]);
            }
            if motor_enabled {
                joint = joint.motor_velocity(motor_target_velocity, 0.0);
            }
            return physics_world.insert_joint(body_handle_1, body_handle_2, joint);
        }
        ImpulseJointHandle::invalid()
    }

    pub fn joint_change_revolute_params(
        &mut self,
        world_handle: WorldHandle,
        joint_handle: ImpulseJointHandle,
        angular_limit_lower: Real,
        angular_limit_upper: Real,
        angular_limit_enabled: bool,
        motor_target_velocity: Real,
        motor_enabled: bool,
    ) {
        if let Some(physics_world) = self.get_mut_world(world_handle)
            && let Some(joint) = physics_world
                .physics_objects
                .impulse_joint_set
                .get_mut(joint_handle)
            && let Some(joint) = joint.data.as_revolute_mut()
        {
            if motor_enabled {
                joint
                    .set_motor_velocity(motor_target_velocity, 0.0)
                    .set_motor_max_force(Real::MAX);
            } else {
                joint.set_motor_velocity(0.0, 0.0).set_motor_max_force(0.0);
            }
            if angular_limit_enabled {
                joint.set_limits([angular_limit_lower, angular_limit_upper]);
            }
        }
    }

    pub fn joint_create_prismatic(
        &mut self,
        world_handle: WorldHandle,
        body_handle_1: RigidBodyHandle,
        body_handle_2: RigidBodyHandle,
        axis: Vector<Real>,
        anchor_1: Vector<Real>,
        anchor_2: Vector<Real>,
        limits: Vector<Real>,
        disable_collision: bool,
    ) -> ImpulseJointHandle {
        if let Some(physics_world) = self.get_mut_world(world_handle) {
            let joint = PrismaticJointBuilder::new(UnitVector::new_unchecked(axis))
                .local_anchor1(Point { coords: anchor_1 })
                .local_anchor2(Point { coords: anchor_2 })
                .limits([limits.x, limits.y])
                .contacts_enabled(!disable_collision);
            return physics_world.insert_joint(body_handle_1, body_handle_2, joint);
        }
        ImpulseJointHandle::invalid()
    }

    pub fn joint_create_spring(
        &mut self,
        world_handle: WorldHandle,
        body_handle_1: RigidBodyHandle,
        body_handle_2: RigidBodyHandle,
        anchor_1: Vector<Real>,
        anchor_2: Vector<Real>,
        stiffness: Real,
        damping: Real,
        rest_length: Real,
        disable_collision: bool,
    ) -> ImpulseJointHandle {
        if let Some(physics_world) = self.get_mut_world(world_handle) {
            let joint = SpringJointBuilder::new(rest_length, stiffness, damping)
                .local_anchor1(Point { coords: anchor_1 })
                .local_anchor2(Point { coords: anchor_2 })
                .contacts_enabled(!disable_collision);
            return physics_world.insert_joint(body_handle_1, body_handle_2, joint);
        }
        ImpulseJointHandle::invalid()
    }

    pub fn joint_change_spring_params(
        &mut self,
        world_handle: WorldHandle,
        joint_handle: ImpulseJointHandle,
        stiffness: Real,
        damping: Real,
        rest_length: Real,
    ) {
        if let Some(physics_world) = self.get_mut_world(world_handle)
            && let Some(joint) = physics_world
                .physics_objects
                .impulse_joint_set
                .get_mut(joint_handle)
        {
            joint
                .data
                .set_motor_position(JointAxis::AngX, rest_length, stiffness, damping);
        }
    }

    pub fn destroy_joint(&mut self, world_handle: WorldHandle, joint_handle: ImpulseJointHandle) {
        if let Some(physics_world) = self.get_mut_world(world_handle) {
            physics_world.remove_joint(joint_handle);
        }
    }

    pub fn joint_change_disable_collision(
        &mut self,
        world_handle: WorldHandle,
        joint_handle: ImpulseJointHandle,
        disable_collision: bool,
    ) {
        if let Some(physics_world) = self.get_mut_world(world_handle)
            && let Some(joint) = physics_world
                .physics_objects
                .impulse_joint_set
                .get_mut(joint_handle)
        {
            joint.data.set_contacts_enabled(!disable_collision);
        }
    }
}
