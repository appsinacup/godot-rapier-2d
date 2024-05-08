use crate::bodies::rapier_collision_object_2d::IRapierCollisionObject2D;
use crate::fluids::rapier_fluid_2d::RapierFluid2D;
use crate::joints::rapier_joint_2d::IRapierJoint2D;
use crate::rapier2d::handle::Handle;
use crate::shapes::rapier_shape_2d::IRapierShape2D;
use crate::spaces::rapier_space_2d::RapierSpace2D;
use godot::builtin::Rid;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub struct RapierShapesSingleton2D {
    pub shapes: HashMap<Rid, Box<dyn IRapierShape2D>>,
}
pub struct RapierSpacesSingleton2D {
    pub spaces: HashMap<Rid, Box<RapierSpace2D>>,
    pub active_spaces: HashMap<Handle, Rid>,
}
pub struct RapierBodiesSingleton2D {
    pub collision_objects: HashMap<Rid, Box<dyn IRapierCollisionObject2D>>,
}
pub struct RapierJointsSingleton2D {
    pub joints: HashMap<Rid, Box<dyn IRapierJoint2D>>,
}
pub struct RapierFluidsSingleton2D {
    pub fluids: HashMap<Rid, Box<RapierFluid2D>>,
}

impl RapierShapesSingleton2D {
    pub fn new() -> RapierShapesSingleton2D {
        RapierShapesSingleton2D {
            shapes: HashMap::new(),
        }
    }
}
impl RapierSpacesSingleton2D {
    pub fn new() -> RapierSpacesSingleton2D {
        RapierSpacesSingleton2D {
            spaces: HashMap::new(),
            active_spaces: HashMap::new(),
        }
    }
}
impl RapierBodiesSingleton2D {
    pub fn new() -> RapierBodiesSingleton2D {
        RapierBodiesSingleton2D {
            collision_objects: HashMap::new(),
        }
    }
}
impl RapierJointsSingleton2D {
    pub fn new() -> RapierJointsSingleton2D {
        RapierJointsSingleton2D {
            joints: HashMap::new(),
        }
    }
}
impl RapierFluidsSingleton2D {
    pub fn new() -> RapierFluidsSingleton2D {
        RapierFluidsSingleton2D {
            fluids: HashMap::new(),
        }
    }
}
unsafe impl Send for RapierShapesSingleton2D {}
unsafe impl Send for RapierSpacesSingleton2D {}
unsafe impl Send for RapierBodiesSingleton2D {}
unsafe impl Send for RapierJointsSingleton2D {}
unsafe impl Send for RapierFluidsSingleton2D {}

pub fn shapes_singleton() -> &'static Mutex<RapierShapesSingleton2D> {
    static HOLDER: OnceLock<Mutex<RapierShapesSingleton2D>> = OnceLock::new();
    HOLDER.get_or_init(|| Mutex::new(RapierShapesSingleton2D::new()))
}

pub fn spaces_singleton() -> &'static Mutex<RapierSpacesSingleton2D> {
    static HOLDER: OnceLock<Mutex<RapierSpacesSingleton2D>> = OnceLock::new();
    HOLDER.get_or_init(|| Mutex::new(RapierSpacesSingleton2D::new()))
}

pub fn bodies_singleton() -> &'static Mutex<RapierBodiesSingleton2D> {
    static HOLDER: OnceLock<Mutex<RapierBodiesSingleton2D>> = OnceLock::new();
    HOLDER.get_or_init(|| Mutex::new(RapierBodiesSingleton2D::new()))
}

pub fn joints_singleton() -> &'static Mutex<RapierJointsSingleton2D> {
    static HOLDER: OnceLock<Mutex<RapierJointsSingleton2D>> = OnceLock::new();
    HOLDER.get_or_init(|| Mutex::new(RapierJointsSingleton2D::new()))
}

pub fn fluids_singleton() -> &'static Mutex<RapierFluidsSingleton2D> {
    static HOLDER: OnceLock<Mutex<RapierFluidsSingleton2D>> = OnceLock::new();
    HOLDER.get_or_init(|| Mutex::new(RapierFluidsSingleton2D::new()))
}
