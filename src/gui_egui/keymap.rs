use crate::common::{ComponentStore, EditorMode, Simulator};
use crate::gui_egui::{editor::Editor, Gui};
use egui::{Key, KeyboardShortcut, Modifiers};
use std::{path::PathBuf, rc::Rc};

#[derive(Copy, Clone)]
pub struct Shortcuts {
    pub file_new: KeyboardShortcut,
    pub file_open: KeyboardShortcut,
    pub file_save: KeyboardShortcut,
    pub file_save_as: KeyboardShortcut,
    pub file_editor_toggle: KeyboardShortcut,
    pub file_preferences: KeyboardShortcut,
    pub file_quit: KeyboardShortcut,
    pub edit_cut: KeyboardShortcut,
    pub edit_copy: KeyboardShortcut,
    pub edit_paste: KeyboardShortcut,
    pub view_zoom_in: KeyboardShortcut,
    pub view_zoom_out: KeyboardShortcut,
    pub control_play_toggle: KeyboardShortcut,
    pub control_play: KeyboardShortcut,
    pub control_pause: KeyboardShortcut,
    pub control_reset: KeyboardShortcut,
    pub control_step_forward: KeyboardShortcut,
    pub control_step_back: KeyboardShortcut,
    pub editor_wire: KeyboardShortcut,
    pub editor_escape: KeyboardShortcut,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self::new()
    }
}

impl Shortcuts {
    pub fn new() -> Self {
        let ctrl = Modifiers {
            alt: false,
            ctrl: true,
            shift: false,
            mac_cmd: false,
            command: false,
        };
        let shift = Modifiers {
            alt: false,
            ctrl: false,
            shift: true,
            mac_cmd: false,
            command: false,
        };
        let none = Modifiers {
            alt: false,
            ctrl: false,
            shift: false,
            mac_cmd: false,
            command: false,
        };

        Shortcuts {
            file_new: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::N,
            },
            file_open: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::O,
            },
            file_save: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::S,
            },
            file_save_as: KeyboardShortcut {
                modifiers: Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: true,
                    mac_cmd: false,
                    command: false,
                },
                key: Key::S,
            },
            file_editor_toggle: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::E,
            },
            file_preferences: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::P,
            },
            file_quit: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::Q,
            },
            edit_cut: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::X,
            },
            edit_copy: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::C,
            },
            edit_paste: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::P,
            },
            view_zoom_in: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::PlusEquals,
            },
            view_zoom_out: KeyboardShortcut {
                modifiers: ctrl,
                key: Key::Minus,
            },
            control_play: KeyboardShortcut {
                modifiers: none,
                key: Key::F6,
            },
            control_play_toggle: KeyboardShortcut {
                modifiers: none,
                key: Key::F5,
            },
            control_pause: KeyboardShortcut {
                modifiers: shift,
                key: Key::F5,
            },
            control_reset: KeyboardShortcut {
                modifiers: Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: true,
                    mac_cmd: false,
                    command: false,
                },
                key: Key::F5,
            },
            control_step_forward: KeyboardShortcut {
                modifiers: none,
                key: Key::F10,
            },
            control_step_back: KeyboardShortcut {
                modifiers: shift,
                key: Key::F10,
            },
            editor_wire: KeyboardShortcut {
                modifiers: none,
                key: Key::W,
            },
            editor_escape: KeyboardShortcut {
                modifiers: none,
                key: Key::Escape,
            },
        }
    }

    pub fn inputs(self, ctx: &egui::Context, gui: &mut Gui) {
        if ctx.input_mut(|i| i.consume_shortcut(&self.file_new)) {
            file_new_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.file_open)) {
            file_open_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.file_save)) {
            file_save_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.file_save_as)) {
            file_save_as_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.file_editor_toggle)) {
            file_editor_toggle_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.file_preferences)) {
            file_preferences_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.file_quit)) {
            file_quit_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.edit_cut)) {
            edit_cut_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.edit_copy)) {
            edit_copy_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.edit_paste)) {
            edit_paste_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.view_zoom_in)) {
            view_zoom_in_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.view_zoom_out)) {
            view_zoom_out_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.control_play_toggle)) {
            control_play_toggle_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.control_play)) {
            control_play_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.control_pause)) {
            control_pause_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.control_reset)) {
            control_reset_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.control_step_forward)) {
            control_step_forward_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.control_step_back)) {
            control_step_back_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.editor_wire)) {
            editor_wire_fn(gui);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.editor_escape)) {
            editor_escape_fn(gui);
        }
    }
}

pub fn file_new_fn(_gui: &mut Gui) {}
pub fn file_open_fn(_gui: &mut Gui) {}
pub fn file_save_fn(gui: &mut Gui) {
    match gui.editor_use {
        true => match gui.editor.as_mut() {
            Some(e) => ComponentStore {
                store: e.components.clone(),
            }
            .save_file(&PathBuf::from("file.json")),
            None => (),
        },
        false => ComponentStore {
            store: gui.simulator.clone().unwrap().ordered_components,
        }
        .save_file(&PathBuf::from("file.json")),
    }
}
pub fn file_save_as_fn(_gui: &mut Gui) {}
pub fn file_editor_toggle_fn(gui: &mut Gui) {
    match gui.editor_use {
        true => {
            gui.editor_use = false;
            match gui.editor.as_mut() {
                Some(e) => {
                    let components = e.components.clone();
                    let simulator =
                        Simulator::new(ComponentStore { store: components }, &mut gui.clock);
                    gui.simulator = Some(simulator);
                }
                _ => (),
            }
            // todo: Make us swap back into simulator mode (create new simulator)
        }
        false => {
            let simulator = std::mem::replace(&mut gui.simulator, None);
            let simulator = simulator.unwrap();
            let mut components = simulator.ordered_components.clone();
            drop(simulator);

            for mut c in components.iter_mut() {
                (*Rc::get_mut(&mut c).unwrap()).set_id_tmp();
            }
            let _ = gui.editor.insert(Editor::gui(components, &gui.path));
            gui.editor_use = true;
        }
    }
}
pub fn file_preferences_fn(_gui: &mut Gui) {}
pub fn file_quit_fn(_gui: &mut Gui) {}
pub fn edit_cut_fn(_gui: &mut Gui) {}
pub fn edit_copy_fn(_gui: &mut Gui) {}
pub fn edit_paste_fn(_gui: &mut Gui) {}
pub fn view_zoom_in_fn(gui: &mut Gui) {
    let scale: &mut f32 = match gui.editor_use {
        true => &mut gui.editor.as_mut().unwrap().scale,
        false => &mut gui.scale,
    };
    match *scale {
        x if (0.0f32..0.2f32).contains(&x) => *scale = 0.25f32,
        x if (0.2f32..0.4f32).contains(&x) => *scale = 0.5f32,
        x if (0.4f32..0.6f32).contains(&x) => *scale = 1f32,
        x if (0.9f32..1.1f32).contains(&x) => *scale = 1.5f32,
        x if (1.4f32..1.6f32).contains(&x) => *scale = 2f32,
        _ => *scale = 2f32,
    }
}
pub fn view_zoom_out_fn(gui: &mut Gui) {
    let scale: &mut f32 = match gui.editor_use {
        true => &mut gui.editor.as_mut().unwrap().scale,
        false => &mut gui.scale,
    };
    match *scale {
        x if (0.2f32..0.4f32).contains(&x) => *scale = 0.1f32,
        x if (0.4f32..0.6f32).contains(&x) => *scale = 0.25f32,
        x if (0.9f32..1.1f32).contains(&x) => *scale = 0.5f32,
        x if (1.4f32..1.6f32).contains(&x) => *scale = 1f32,
        x if (1.9f32..2.1f32).contains(&x) => *scale = 1.5f32,
        _ => *scale = 0.1f32,
    }
}
pub fn control_play_toggle_fn(gui: &mut Gui) {
    if !gui.editor_use {
        gui.pause = !gui.pause;
    }
}
pub fn control_play_fn(gui: &mut Gui) {
    if !gui.editor_use {
        gui.pause = false;
    }
}
pub fn control_pause_fn(gui: &mut Gui) {
    if !gui.editor_use {
        gui.pause = true;
    }
}
pub fn control_reset_fn(gui: &mut Gui) {
    if !gui.editor_use {
        gui.simulator.as_mut().unwrap().reset(&mut gui.clock);
        gui.pause = true;
    }
}
pub fn control_step_forward_fn(gui: &mut Gui) {
    if !gui.editor_use {
        gui.simulator.as_mut().unwrap().clock(&mut gui.clock);
    }
}
pub fn control_step_back_fn(gui: &mut Gui) {
    if !gui.editor_use {
        gui.simulator.as_mut().unwrap().un_clock(&mut gui.clock);
    }
}
pub fn editor_wire_fn(gui: &mut Gui) {
    if gui.editor_use {
        let editor = gui.editor.as_mut().unwrap();
        match editor.editor_mode {
            EditorMode::Default | EditorMode::Input => {
                editor.editor_mode = EditorMode::Wire;
            }
            EditorMode::Wire => {
                editor.editor_mode = EditorMode::Default;
            }
        }
        crate::gui_egui::editor_wire::reset_wire_mode(editor);
    }
}
pub fn editor_escape_fn(gui: &mut Gui) {
    if gui.editor_use {
        let editor = gui.editor.as_mut().unwrap();
        editor.editor_mode = EditorMode::Default;
        crate::gui_egui::editor_wire::reset_wire_mode(editor);
        crate::gui_egui::library::reset_input_mode(editor);
    }
}
