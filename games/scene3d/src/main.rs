//! 3D Scene Editor — validates Canvas3D + MCP scene tools for AI agent map editing.
//!
//! This game:
//! 1. Loads a scene from "levels/test.ron" (if exists), or starts empty
//! 2. Renders scene objects via Canvas3D
//! 3. Captures frame 3 for AI visual feedback
//! 4. Supports MCP-driven editing (spawn_object, set_component, save_scene)

use anvilkit::prelude::*;
use anvilkit::core::time::Time;
use anvilkit_app::{AnvilKitApp, GameCallbacks, GameConfig, GameContext};
use anvilkit_render::renderer::canvas3d::{Canvas3D, Canvas3DRenderer};
use anvilkit_mcp::scene::{SceneObject, SceneTransform, Shape, SerializedScene};
use glam::Vec3;

struct SceneEditorGame {
    renderer: Option<Canvas3DRenderer>,
    frame: u32,
    angle: f32,
}

impl GameCallbacks for SceneEditorGame {
    fn init(&mut self, ctx: &mut GameContext) {
        let Some(device) = ctx.render_app.render_device() else { return };
        let Some(format) = ctx.render_app.surface_format() else { return };
        let (w, h) = ctx.render_app.window_state().size();
        self.renderer = Some(Canvas3DRenderer::new(device, format, w, h));

        // Spawn some default objects if no MCP connected
        let world = ctx.app.world_mut();
        world.spawn((
            SceneObject { shape: Shape::Cube { size: [1.0, 1.0, 1.0] }, color: [0.9, 0.2, 0.15, 1.0], name: "red_cube".into() },
            SceneTransform { translation: [-2.0, 0.5, 0.0], ..Default::default() },
        ));
        world.spawn((
            SceneObject { shape: Shape::Sphere { radius: 1.0 }, color: [0.15, 0.4, 0.9, 1.0], name: "blue_sphere".into() },
            SceneTransform { translation: [2.0, 1.0, 0.0], ..Default::default() },
        ));
        world.spawn((
            SceneObject { shape: Shape::Cube { size: [1.5, 1.5, 1.5] }, color: [0.95, 0.85, 0.1, 1.0], name: "yellow_cube".into() },
            SceneTransform { translation: [0.0, 0.75, -2.5], ..Default::default() },
        ));
        world.spawn((
            SceneObject { shape: Shape::Ground { size: 20.0 }, color: [0.2, 0.25, 0.2, 1.0], name: "ground".into() },
            SceneTransform::default(),
        ));

        // Save the default scene
        let scene = SerializedScene::from_world(ctx.app.world_mut());
        let _ = scene.save("levels/test_scene.ron");
        println!(">>> Default scene saved to levels/test_scene.ron ({} objects)", scene.objects.len());
    }

    fn update(&mut self, ctx: &mut GameContext) {
        let dt = ctx.app.world().resource::<Time>().delta_seconds();
        self.angle += dt * 0.3;
    }

    fn render(&mut self, ctx: &mut GameContext) {
        let Some(ref mut renderer) = self.renderer else { return };
        let Some(mut c) = Canvas3D::begin(ctx.render_app, renderer) else { return };

        // Camera orbit
        let cam_x = self.angle.cos() * 10.0;
        let cam_z = self.angle.sin() * 10.0;
        c.set_camera(Vec3::new(cam_x, 6.0, cam_z), Vec3::ZERO, 60.0);
        c.set_sun(Vec3::new(-1.0, -1.0, -0.5), [1.0, 0.95, 0.85]);
        c.clear([0.05, 0.05, 0.1, 1.0]);

        // Render all SceneObjects from the ECS world
        {
            let world = ctx.app.world_mut();
            let mut query = world.query::<(&SceneObject, &SceneTransform)>();
            for (obj, transform) in query.iter(world) {
                let pos = Vec3::from(transform.translation);
                match &obj.shape {
                    Shape::Cube { size } => c.draw_cube(pos, Vec3::from(*size), obj.color),
                    Shape::Sphere { radius } => c.draw_sphere(pos, *radius, obj.color),
                    Shape::Ground { size } => c.draw_ground(*size, obj.color),
                }
            }
        }

        // Capture frame 3
        self.frame += 1;
        if self.frame == 3 {
            c.capture_frame("screenshots/scene_editor.png");
            println!(">>> Scene captured to screenshots/scene_editor.png");
        }

        c.finish();
    }
}

fn main() {
    println!("Scene Editor — Canvas3D + MCP scene tools test");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins::new().with_window(
        WindowConfig::new().with_title("Scene Editor").with_size(800, 600),
    ));

    AnvilKitApp::run(
        GameConfig::new("Scene Editor").with_size(800, 600),
        app,
        SceneEditorGame { renderer: None, frame: 0, angle: 0.0 },
    );
}
