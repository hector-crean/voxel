use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use compute_mesh::channels::MainWorldReceiver;
use compute_mesh::GpuReadbackPlugin;

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    option: f32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, GpuReadbackPlugin))
        .add_plugins(
            ResourceInspectorPlugin::<Configuration>::default()
                .run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .init_resource::<Configuration>()
        .register_type::<Configuration>()
        .add_systems(Update, MainWorldReceiver::receive)
        .run();
}
