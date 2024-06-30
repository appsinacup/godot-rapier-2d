# 1. Documentation

1. [Rapier Physics Server](#rapier-physics-server)
2. [Rapier Direct Space State](#rapier-direct-space-state)

## Rapier Physics Server

The new `RapierPhysicsServer2D` and `RapierPhysicsServer3D` can be used just like the `PhysicsServer2D` and `PhysicsServer3D` in GDScript:
```python
RapierPhysicsServer2D.body_set_extra_param(...)
```

The rapeir physics server exposes new methods that expose extra functionality that it has to offer, such as:
- bodies rounded edges
- fluids
- json and binary export

## Rapier Direct Space State

Same as the Rapier Physics Server, the Rapier Direct Space state offeres new functions that allow for functionality such as:
- json and binary export

# 2. Implementation

1. [Extending Godot Physics](#extending-godot-physics)
2. [How the Physics Server works with Godot Nodes](#how-the-physics-server-works-with-godot-nodes)
3. [Rapier Physics Server Serialization](#rapier-physics-server-serialization)
4. [Rapier Physics Server Fluids](#rapier-physics-server-fluids)
5. [How to build](#how-to-build)
6. [How to debug](#how-to-debug)

-----

## Extending Godot Physics

Godot let's any GDExtension create a new `Physics Server` if they call `register_server` on the [PhysicsServer2DManager](https://docs.godotengine.org/en/latest/classes/class_physicsserver2dmanager.html) or [PhysicsServer3DManager](https://docs.godotengine.org/en/latest/classes/class_physicsserver3dmanager.html) and on the callback return a new instance of their custom `Physics Server` that extends either [PhysicsServer2DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsserver2dextension.html) or [PhysicsServer3DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsserver3dextension.html).

A physics server has to implement 3 classes:
- [PhysicsServer2DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsserver2dextension.html) or [PhysicsServer3DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsserver3dextension.html)
- [PhysicsDirectBodyState2DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsdirectbodystate2dextension.html) or [PhysicsDirectBodyState3DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsdirectbodystate3dextension.html)
- [PhysicsDirectSpaceState2DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsdirectspacestate2dextension.html) or [PhysicsDirectSpaceState3DExtension](https://docs.godotengine.org/en/latest/classes/class_physicsdirectspacestate3dextension.html)

Note that these functions all have underscore in begining of function name. The flow for how Godot calls them is the function without underscore calls into th one with underscore, eg.:

```c++
body_create()
|
v
_body_create()
```

Godot can register multiple `Physics Servers`, and can decide at runtime which one to use.

Godot Rapier Physics implements these under the name like this:
- [RapierPhysicsServer2D](../src/servers/rapier_physics_server_2d.rs) and [RapierPhysicsServer3D](../src/servers/rapier_physics_server_3d.rs)
- [RapierDirectBodyState2D](../src/bodies/rapier_direct_body_state_2d.rs) and [RapierDirectBodyState3D](../src/bodies/rapier_direct_body_state_3d.rs)
- [RapierDirectSpaceState2D](../src/spaces/rapier_direct_space_state_2d.rs) and [RapierDirectSpaceState3D](../src/spaces/rapier_direct_space_state_3d.rs)

Code is reused as much as possible, by having each of these classes call into a generic class that does the implementation, and a few configuration flags to select dimensions, precision, etc.

The code written here is based on the Physics Server from Godot.

## How the Physics Server works with Godot Nodes

The `Physics Server` is a singleton with a set of API's for bodies, areas, shapes and spaces. An example of this is [body_create](https://docs.godotengine.org/en/latest/classes/class_physicsserver2d.html#class-physicsserver2d-method-body-create) function. This function creates a body, and returns an [RID](https://docs.godotengine.org/en/latest/classes/class_rid.html#class-rid). These `RID`'s are resource id's for objects that are handled by the `Physics Server`.

Normally you wouldn't call these functions directly, but instead you would create a node that would call these functions. Eg. when you create a `RigidBody2D` node, this calls the following:
```c++
body_rid = body_create()
body_set_space(body_rid, space_rid)
body_set_state(body_rid, BODY_STATE_TRANSFORM, Transform2D())
...
```

Also, aside from communication from Godot -> `Physics Server`, there is also the reverse, where the `Physics Server` notifies Godot of updates (eg. collision events, etc.). For this, Godot first calls into the `Physics Server`'s method, eg.:
- [body_set_force_integration_callback](https://docs.godotengine.org/en/latest/classes/class_physicsserver2d.html#class-physicsserver2d-method-body-set-force-integration-callback)
- [area_set_monitor_callback](https://docs.godotengine.org/en/latest/classes/class_physicsserver2d.html#class-physicsserver2d-method-area-set-area-monitor-callback)
- [area_set_monitor_callback](https://docs.godotengine.org/en/latest/classes/class_physicsserver2d.html#class-physicsserver2d-method-area-set-monitor-callback)

These functions send a `Callable` to the `Physics Server` that it uses to call back into Godot and notify of certain changes.

## Rapier Physics Server Serialization

The `Rapier Physics Server`, which is our custom implementation of `Physics Server`, hands out `RID`'s for objects it has internally, similar to how `Godot Physics Server` does. The classes are named similar to how Godot naming works, but with Rapier instead of Godot, eg.:
- `RapierArea`
- `RapierBody`
- `RapierCollisionObject`
- ...

The difference from `Godot Physics Server` is that resources are not using pointers, and as such are safe to be serialized.

However, there are some things that need to be reconstructed after saving and loading the state, namely:
- `Callables`
- `RID`
- `instance_id`
- `canvas_instance_id`

As described in the chapter above, the `Physics Server` creates some resources and gives Godot `RID`'s that can be used to access them. Godot also gives some resources to the `Physics Server`, eg. `Callables`, that are used to notify Godot of updates (eg. collision events, etc.). Godot also gives to objects from `Physics Server` instance_id's and canvas_instance_id's in order to identify a node corresponds to a resource.

As such, when saving and loading, first one must recreate the Godot scene and make sure all the dependencies are created, and only later load the `Physics Server` state. Even if some of the things the Physics Server loads won't be the exact same, eg. `RID`'s, `Callables`, etc. The simulation will work deterministic as internally it uses Rapier which is itself deterministic. Also the update order isn't based on these resources that Godot hands over.

## Rapier Physics Server Fluids

Godot Rapier Physics also exposes new nodes, [Fluid2D](../src/fluids/fluid_2d.rs) and [Fluid3D](../src/fluids/fluid_3d.rs). These nodes can set and get position, velocity and acceleration of particles used for fluid simulation. Internally it uses [salva](https://github.com/dimforge/salva) to create them.

## How to build

1. Prerequisites:
- [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

2. Update dependencies to latest:

```bash
cargo update
```

3. Build the project
```bash
cargo build --release
```

By default it builds for 2d, if you want to specify additional build features, you can do so with the `--features` flag:

```bash
cargo build --release --features="single-dim3,parallel,simd-stable,serde-serialize" --no-default-features
```

Note the `--no-default-features`, because by default it has the `single-dim2` feature

4. Copy the output to bin folder of the addon:

Eg. macOS
```bash
cp target/release/libgodot_rapier.dylib bin2/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
```
Eg. Windows
```bash
cp target/release/godot_rapier.dll bin2/addons/godot-rapier2d/bin/godot_rapier.windows.x86_64-pc-windows-msvc.dll
```

For the correct path to use inside the bin folder, look inside the `bin2d/addons/godot-rapier2d.gdextension` or the `bin3d/addons/godot-rapier3d.gdextension`.

### Available features

For features, the following are available:
- `single-dim2`
- `single-dim3`
- `double-dim2`
- `double-dim3`
- `parallel`
- `simd-stable`
- `enhanced-determinism`
- `serde-serialize`

The `single` and `double` refer to the [precision](https://docs.godotengine.org/en/stable/tutorials/physics/large_world_coordinates.html) used in Godot.

The `dim2` or `dim3` refers to the rapier and salva versions used, and to what classes will be implemented for Godot and what Physics Server will be replaced(2d or 3d).

The `parallel` version doesn't work on web.

The `enhanced-determinism` usually slows down the simulation.

The `serde-serialize` feature enabled serialization methods for physics server.

## How to debug

### Run Godot from debugger inside VSCode

Click `Run and Debug` in VSCode on the left. Add a configuration to the `launch.configurations` similar to this (eg. below one is for macOS):
```json
{
    "name": "Launch",
    "program": "path/to/godot/bin/godot.macos.editor.dev.arm64",
    "type": "cppdbg",
    "request": "launch",
    "cwd": "${workspaceFolder:godot-rapier-2d}",
    "osx": {
        "MIMode": "lldb"
    },
    "args": [
        "--path",
        "path/to/project/folder",
        "--debug-collisions",
        "scene-name.tscn"
    ]
},
```

More on godot cli usage [here](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html).
