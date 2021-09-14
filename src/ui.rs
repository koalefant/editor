use crate::app::App;
use crate::document::{ChangeMask, TraceMethod, Document};
use crate::tool::Tool;
use anyhow::Context;
use rimui::*;
use std::path::{Path, PathBuf};

impl App {
    pub fn ui(&mut self, context: &mut miniquad::Context, _time: f32, dt: f32) {
        self.ui_toolbar(context);

        self.ui_sidebar(context);

        self.ui_error_message(context);

        self.ui.layout_ui(
            dt,
            [0, 0, self.window_size[0] as i32, self.window_size[1] as i32],
            None,
        );
    }

    fn ui_sidebar(&mut self, context: &mut miniquad::Context) {
        let window = self.ui.window(
            "Test",
            WindowPlacement::Absolute {
                pos: [self.window_size[0] as i32 - 4, 4],
                size: [0, self.window_size[1] as i32 - 8],
                expand: EXPAND_LEFT,
            },
            0,
            0,
        );

        let frame = self.ui.add(window, Frame::default());
        let rows = self.ui.add(frame, vbox().padding(2).min_size([200, 0]));
        self.ui.add(rows, label("Layers"));
        self.ui
            .add(rows, button("0. Grid").down(true).align(Some(Align::Left)));
        {
            let trace_method = self.doc.borrow().layer.trace_method;
            self.ui.add(rows, label("Trace Method"));
            let mut new_trace_method = None;
            if self
                .ui
                .add(
                    rows,
                    button("Walk").down(matches!(trace_method, TraceMethod::Walk)),
                )
                .clicked
            {
                new_trace_method = Some(TraceMethod::Walk);
            }
            if self
                .ui
                .add(
                    rows,
                    button("Grid").down(matches!(trace_method, TraceMethod::Grid)),
                )
                .clicked
            {
                new_trace_method = Some(TraceMethod::Grid);
            }
            if let Some(new_trace_method) = new_trace_method {
                let mut doc = self.doc.borrow_mut();
                doc.layer.trace_method = new_trace_method;
                self.dirty_mask.cells = true;
            }
        }

        self.ui.add(rows, label("Reference"));
        if self.doc.borrow().reference_path.is_some() {
            let show_reference = self.doc.borrow().show_reference;
            if self
                .ui
                .add(rows, button("Show Reference").down(show_reference))
                .clicked
            {
                self.doc.borrow_mut().show_reference = !show_reference;
            }
        }

        let mut doc = self.doc.borrow_mut();
        let mut buffer = String::new();
        let reference_text = doc
            .reference_path
            .as_ref()
            .map(|s| {
                if let Some((_, name)) = s.rsplit_once('/') {
                    buffer = format!(".../{}", name);
                    &buffer
                } else {
                    s.as_str()
                }
            })
            .unwrap_or("Load...");

        if self.ui.add(rows, button(reference_text)).clicked {
            let mut new_reference_path = self.report_error({
                let path = doc.reference_path.as_ref().map(PathBuf::from);
                nfd2::open_file_dialog(Some("png"), path.as_ref().map(|p| p.as_path()))
                    .context("Opening dialog")
            });

            if let Some(nfd2::Response::Okay(new_reference_path)) = new_reference_path {
                doc.reference_path = Some(new_reference_path.to_string_lossy().to_string());
                self.graphics.borrow_mut().generate(
                    &doc,
                    ChangeMask {
                        reference_path: true,
                        ..ChangeMask::default()
                    },
                    Some(context),
                );
            }
        }
        if let Some(path) = &doc.reference_path {
            last_tooltip(&mut self.ui, rows, path);
        }

        drop(doc);
    }

    pub fn ui_toolbar(&mut self, context: &mut miniquad::Context) {
        let toolbar = self.ui.window(
            "Map",
            WindowPlacement::Absolute {
                pos: [4, 4],
                size: [0, 32],
                expand: EXPAND_RIGHT,
            },
            0,
            0,
        );
        {
            let frame = self.ui.add(toolbar, Frame::default());
            let cols = self.ui.add(frame, hbox());
            self.ui.add(cols, label("Map"));
            if self.ui.add(cols, button("Open")).clicked {
                let response =
                    self.report_error(nfd2::open_file_dialog(None, None).context("Opening dialog"));
                if let Some(nfd2::Response::Okay(path)) = response {
                    let doc = self.report_error(App::load_doc(&path));
                    if let Some(doc) = doc {
                        self.doc.replace(doc);
                        self.doc_path = Some(path);
                        let state_res = self.save_app_state();
                        self.report_error(state_res);
                    }
                };
            }
            let mut save_as = false;
            if self.ui.add(cols, button("Save")).clicked {
                if let Some(path) = &self.doc_path {
                    self.report_error(App::save_doc(path, &self.doc.borrow(), &self.view));
                    let state_res = self.save_app_state();
                    self.report_error(state_res);
                } else {
                    save_as = true;
                }
            }
            if self.ui.add(cols, button("Save As...")).clicked {
                save_as = true;
            }

            if save_as {
                let path = self.report_error(
                    nfd2::open_save_dialog(Some("js"), None).context("Opening dialog"),
                );

                if let Some(nfd2::Response::Okay(path)) = path {
                    self.report_error(App::save_doc(
                        Path::new(&path),
                        &self.doc.borrow(),
                        &self.view,
                    ));
                    let state_res = self.save_app_state();
                    self.report_error(state_res);
                }
            }

            self.ui.add(cols, label("Edit"));
            if self.ui.add(cols, button("Undo").enabled(!self.undo.is_empty())).clicked ||
                //self.ui.key_pressed_with_modifiers(KeyCode::Z, true, false, false) {
                self.ui.key_pressed(KeyCode::Z) {
                let mut doc_ref = self.doc.borrow_mut();
                let doc: &mut Document = &mut doc_ref;
                let err = self.undo.apply(doc, &mut self.redo);
                self.report_error(err);
                self.dirty_mask = ChangeMask{ cells: true, reference_path: false };
            }
            if self.ui.add(cols, button("Redo").enabled(!self.redo.is_empty())).clicked ||
                //self.ui.key_pressed_with_modifiers(KeyCode::Z, true, true, false)
                self.ui.key_pressed(KeyCode::Y) {
                let mut doc_ref = self.doc.borrow_mut();
                let doc: &mut Document = &mut doc_ref;
                let err = self.redo.apply(doc, &mut self.undo);
                self.report_error(err);
                self.dirty_mask = ChangeMask{ cells: true, reference_path: false };
            }


            self.ui.add(cols, label("Tool"));

            let tool = self.tool.clone();
            if self
                .ui
                .add(cols, button("Pan").down(matches!(tool, Tool::Pan { .. })))
                .clicked
            {
                self.tool = Tool::Pan;
            }
            if self
                .ui
                .add(
                    cols,
                    button("Paint").down(matches!(tool, Tool::Paint { .. })),
                )
                .clicked
            {
                self.tool = Tool::Paint;
            }
        }
    }

    fn ui_error_message(&mut self, context: &mut miniquad::Context) {
        let error_message_borrow = self.error_message.borrow();
        if let Some(error_message) = error_message_borrow.as_ref() {
            let window = self.ui.window(
                "ErrorMessage",
                WindowPlacement::Center {
                    size: [0, 0],
                    offset: [0, 0],
                    expand: EXPAND_ALL,
                },
                0,
                0,
            );

            let frame = self.ui.add(window, Frame::default());
            let rows = self.ui.add(
                frame,
                vbox().padding(2).min_size([200, 0]).margins([8, 8, 8, 8]),
            );
            self.ui.add(
                rows,
                wrapped_text("message", &error_message)
                    .min_size([300, 0])
                    .max_width(500),
            );
            let columns = self.ui.add(rows, hbox());

            self.ui.add(columns, spacer());
            drop(error_message_borrow);
            if self
                .ui
                .add(columns, button("OK").min_size([120, 0]))
                .clicked
            {
                self.error_message.replace(None);
            }
            self.ui.add(columns, spacer());
        }
    }
}

fn last_tooltip(ui: &mut UI, parent: AreaRef, tooltip_text: &str) {
    if let Some(t) = ui.last_tooltip(
        parent,
        Tooltip {
            placement: TooltipPlacement::Beside,
            ..Tooltip::default()
        },
    ) {
        let frame = ui.add(t, Frame::default());
        let rows = ui.add(frame, vbox());
        ui.add(rows, label(tooltip_text));
    }
}