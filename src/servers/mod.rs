use godot::prelude::*;

use crate::servers::rapier_project_settings::RapierProjectSettings;
#[cfg(feature = "dim2")]
pub mod rapier_physics_server_2d;
#[cfg(feature = "dim3")]
pub mod rapier_physics_server_3d;
pub mod rapier_physics_server_extra;
pub mod rapier_physics_server_impl;
pub mod rapier_project_settings;
#[cfg(feature = "dim2")]
pub type RapierPhysicsServer = rapier_physics_server_2d::RapierPhysicsServer2D;
#[cfg(feature = "dim3")]
pub type RapierPhysicsServer = rapier_physics_server_3d::RapierPhysicsServer3D;
#[cfg(feature = "dim2")]
pub fn register_server() {
    use godot::engine::PhysicsServer2DManager;
    let mut manager = PhysicsServer2DManager::singleton();
    let factory =
        crate::servers::rapier_physics_server_2d::RapierPhysicsServerFactory2D::new_alloc();
    manager.register_server("Rapier2D".into(), factory.callable("create_server"));
}
#[cfg(feature = "dim3")]
pub fn register_server() {
    use godot::engine::PhysicsServer3DManager;
    let mut manager = PhysicsServer3DManager::singleton();
    let factory =
        crate::servers::rapier_physics_server_3d::RapierPhysicsServerFactory3D::new_alloc();
    manager.register_server("Rapier3D".into(), factory.callable("create_server"));
}
pub fn register_scene() {
    RapierProjectSettings::register_settings();
}
pub fn unregister_server() {
}
pub fn unregister_scene() {}
pub fn register_editor() {
    // TODO register documentation
    //GDExtensionsInterfaceEditorHelpLoadXMLAndLen editor_help_load_xml_from_utf8_chars_and_len = (GDExtensionsInterfaceEditorHelpLoadXMLAndLen)internal::gdextension_interface_get_proc_address("editor_help_load_xml_from_utf8_chars_and_len");
    //editor_help_load_xml_from_utf8_chars_and_len(_doc_data, _doc_data_size);
}
pub fn unregister_editor() {
    
}
