mod app;
mod document;
mod graphics;
mod interaction;
mod math;
mod tool;
mod ui;
mod undo_stack;

use crate::document::ChangeMask;
use crate::math::critically_damped_spring;
use app::*;
use core::default::Default;
use glam::vec2;
use miniquad::{conf, Context, EventHandler, PassAction, UserData};
use rimui::*;

impl EventHandler for App {
    fn update(&mut self, context: &mut Context) {
        let time = (miniquad::date::now() - self.start_time) as f32;
        let dt = time - self.last_time;

        critically_damped_spring(
            &mut self.view.zoom,
            &mut self.view.zoom_velocity,
            self.view.zoom_target,
            dt,
            0.125,
        );

        self.ui(context, time, dt);

        if self.dirty_mask != ChangeMask::default() {
            self.graphics
                .borrow_mut()
                .generate(&self.doc.borrow(), self.dirty_mask, Some(context));
            self.dirty_mask = ChangeMask::default();
        }

        self.last_time = time;
    }

    fn draw(&mut self, context: &mut Context) {
        let _time = (miniquad::date::now() - self.start_time) as f32;
        context.begin_default_pass(PassAction::Clear {
            color: Some((0.2, 0.2, 0.2, 1.0)),
            depth: None,
            stencil: None,
        });

        self.batch.begin_frame();
        self.batch.clear();
        let g = self.graphics.borrow();
        self.batch.set_image(self.white_texture);
        let screen_origin = self.document_to_screen(vec2(0.0, 0.0));
        self.batch
            .geometry
            .fill_circle_aa(screen_origin, 4.0, 4, [255, 255, 255, 255]);

        if self.doc.borrow().show_reference {
            if let Some(reference) = g.reference_texture {
                let w = reference.width;
                let h = reference.height;

                let t = self.view.world_to_screen();

                let p0 = t.transform_point2(vec2(0.0, 0.0));
                let p1 = t.transform_point2(vec2(w as f32, h as f32));

                self.batch.set_image(reference);
                self.batch.geometry.fill_rect_uv(
                    [p0.x, p0.y, p1.x, p1.y],
                    [0.0, 0.0, 1.0, 1.0],
                    [255, 255, 255, 255],
                );
            }
        }

        // actual map drawing
        self.batch.set_image(self.white_texture);
        self.graphics.borrow().draw(&mut self.batch, &self.view);

        let white_texture = self.white_texture.clone();
        let mut render = MiniquadRender::new(&mut self.batch, &self.font_manager, |_sprite_key| {
            white_texture.clone()
        });
        self.ui.render_ui(&mut render, None);

        context.apply_pipeline(&self.pipeline);
        context.apply_uniforms(&ShaderUniforms {
            screen_size: self.window_size,
        });
        self.batch.flush(context);

        context.end_render_pass();

        context.commit_frame();
    }

    fn resize_event(&mut self, _context: &mut Context, width: f32, height: f32) {
        self.window_size = [width, height];
        self.view.screen_width_px = width - 200.0;
        self.view.screen_height_px = height;
    }

    fn mouse_motion_event(&mut self, _c: &mut miniquad::Context, x: f32, y: f32) {
        self.last_mouse_pos = [x, y];

        self.handle_event(UIEvent::MouseMove {
            pos: [x as i32, y as i32],
        });
    }

    fn mouse_wheel_event(&mut self, _c: &mut miniquad::Context, _dx: f32, dy: f32) {
        self.handle_event(UIEvent::MouseWheel {
            pos: [self.last_mouse_pos[0] as i32, self.last_mouse_pos[1] as i32],
            delta: dy,
        });
    }

    fn mouse_button_down_event(
        &mut self,
        _c: &mut miniquad::Context,
        button: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.handle_event(UIEvent::MouseDown {
            pos: [x as i32, y as i32],
            button: ui_mouse_button(button),
            time: miniquad::date::now(),
        });
    }

    fn mouse_button_up_event(
        &mut self,
        _c: &mut miniquad::Context,
        button: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.handle_event(UIEvent::MouseUp {
            pos: [x as i32, y as i32],
            button: ui_mouse_button(button),
        });
    }

    fn char_event(
        &mut self,
        _c: &mut miniquad::Context,
        character: char,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        if !keymods.ctrl {
            self.handle_event(UIEvent::TextInput {
                text: character.to_string(),
            });
        }
    }

    fn key_down_event(
        &mut self,
        _c: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        repeat: bool,
    ) {

        let is_always_consumed = match (keycode, keymods.ctrl, keymods.shift, keymods.alt) {
            (miniquad::KeyCode::Z, _, _, _) => true,
            (miniquad::KeyCode::Y, _, _, _) => true,
            (miniquad::KeyCode::PageDown | miniquad::KeyCode::PageUp, _, _, _) => true,
            _ => false
        };
        if self.ui.consumes_key_down() || is_always_consumed {
            let ui_keycode = match keycode {
                miniquad::KeyCode::Enter | miniquad::KeyCode::KpEnter => Some(KeyCode::Enter),
                miniquad::KeyCode::Left => Some(KeyCode::Left),
                miniquad::KeyCode::Right => Some(KeyCode::Right),
                miniquad::KeyCode::Up => Some(KeyCode::Up),
                miniquad::KeyCode::Down => Some(KeyCode::Down),
                miniquad::KeyCode::Home => Some(KeyCode::Home),
                miniquad::KeyCode::End => Some(KeyCode::End),
                miniquad::KeyCode::PageUp => Some(KeyCode::PageUp),
                miniquad::KeyCode::PageDown => Some(KeyCode::PageDown),
                miniquad::KeyCode::Delete => Some(KeyCode::Delete),
                miniquad::KeyCode::Backspace => Some(KeyCode::Backspace),
                miniquad::KeyCode::Z => Some(KeyCode::Z),
                miniquad::KeyCode::X => Some(KeyCode::X),
                miniquad::KeyCode::C => Some(KeyCode::C),
                miniquad::KeyCode::V => Some(KeyCode::V),
                miniquad::KeyCode::Y => Some(KeyCode::Y),
                miniquad::KeyCode::A => Some(KeyCode::A),
                _ => None,
            };

            if let Some(ui_keycode) = ui_keycode {
                let event = UIEvent::KeyDown {
                    key: ui_keycode,
                    control: keymods.ctrl,
                    shift: keymods.shift,
                    alt: keymods.alt,
                };
                let render_rect = [0, 0, self.window_size[0] as i32, self.window_size[1] as i32];
                if self
                    .ui
                    .handle_event(&event, render_rect, miniquad::date::now() as f32)
                {
                }
            }
            return;
        }

        if repeat {
            return;
        }
    }
}

fn ui_mouse_button(button: miniquad::MouseButton) -> i32 {
    match button {
        miniquad::MouseButton::Left => 1,
        miniquad::MouseButton::Right => 2,
        miniquad::MouseButton::Middle => 3,
        miniquad::MouseButton::Unknown => 4,
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let current_exe = std::env::current_exe().expect("missing exe path");
        let mut resources_path = current_exe
            .parent()
            .expect("cannot serve from the root")
            .to_path_buf();
        loop {
            let in_target = resources_path.ends_with("target");
            if !resources_path.pop() {
                panic!(
                    "cannot find target in the exe path {}",
                    current_exe.to_str().expect("unprintable chars in path")
                );
            }
            if in_target {
                resources_path.push("res");
                break;
            }
        }
        std::env::set_current_dir(&resources_path).expect("failed to set current directory");
    }

    #[cfg(not(target_arch = "wasm32"))]
    simple_logger::SimpleLogger::new()
        .with_module_level("ws", log::LevelFilter::Warn)
        .with_module_level("mio", log::LevelFilter::Warn)
        .init()
        .unwrap();

    miniquad::start(
        conf::Conf {
            sample_count: 0,
            window_width: 1280,
            window_height: 720,
            ..Default::default()
        },
        |mut context| UserData::owning(App::new(&mut context), context),
    );
}
