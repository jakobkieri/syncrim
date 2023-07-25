use crate::common::{InputPort, Ports};
use crate::gui_egui::editor::EditorMode;
use crate::gui_egui::helper::{editor_mode_to_sense, out_of_bounds, unique_component_name};
use egui::{ComboBox, Frame, Margin, PointerButton, Pos2, Rect, Response, Rounding, Ui, Window};
use epaint::Shadow;

pub fn rect_with_hover<P>(
    rect: Rect,
    clip_rect: Rect,
    editor_mode: EditorMode,
    ui: &mut Ui,
    id: String,
    f: P,
) -> Response
where
    P: Fn(&mut Ui),
{
    let rect = out_of_bounds(rect, clip_rect);
    let r = ui.allocate_rect(rect, editor_mode_to_sense(editor_mode));

    if r.hovered() && !r.dragged() {
        egui::containers::popup::show_tooltip_for(ui.ctx(), egui::Id::new(id), &rect, |ui| {
            f(ui);
        });
    }
    r
}

pub fn properties_window<P>(
    ui: &mut Ui,
    id: String,
    resp: &Response,
    properties_window: &mut bool,
    mut f: P,
) where
    P: FnMut(&mut Ui),
{
    if *properties_window {
        let resp = Window::new(format!("Properties: {}", id))
            .frame(Frame {
                inner_margin: Margin::same(10f32),
                outer_margin: Margin::same(0f32),
                rounding: Rounding::same(10f32),
                shadow: Shadow::small_dark(),
                fill: ui.visuals().panel_fill,
                stroke: ui.visuals().window_stroke,
            })
            .default_pos(Pos2 {
                x: (resp.rect.min.x + resp.rect.max.x) / 2f32,
                y: (resp.rect.min.y + resp.rect.max.y) / 2f32,
            })
            .show(ui.ctx(), |ui| {
                f(ui);
            });
        if resp.unwrap().response.clicked_elsewhere() {
            *properties_window = false;
        }
    }
    if resp.clicked_by(PointerButton::Secondary) {
        // Open properties window
        *properties_window = true;
    }
}

pub fn pos_slider(ui: &mut Ui, pos: &mut (f32, f32)) {
    ui.horizontal(|ui| {
        ui.add(
            egui::Slider::new(&mut (*pos).0, 0f32..=1000f32)
                .text("pos x")
                .clamp_to_range(false),
        );
        ui.add(
            egui::Slider::new(&mut (*pos).1, 0f32..=1000f32)
                .text("pos y")
                .clamp_to_range(false),
        );
    });
}
pub fn input_selector(
    ui: &mut Ui,
    input_port: &mut InputPort,
    id_ports: &Vec<(crate::common::Id, Ports)>,
) {
    let mut port_id = input_port.input.id.clone();
    let mut port_field = input_port.input.field.clone();
    let label_port_id = format!("{}.id", input_port.port_id.clone());
    let text_port_id = format!("{}", port_id);
    let label_port_field = format!("{}.field", input_port.port_id.clone());
    let text_port_field = format!("{}", port_field);
    ui.horizontal(|ui| {
        ComboBox::from_label(label_port_id)
            .selected_text(text_port_id)
            .show_ui(ui, |ui| {
                for c in id_ports {
                    let id = c.0.clone();
                    ui.selectable_value(&mut port_id, id.clone(), id);
                }
            });
        ComboBox::from_label(label_port_field)
            .selected_text(text_port_field)
            .show_ui(ui, |ui| {
                for c in id_ports {
                    let id = c.0.clone();
                    if id != port_id {
                        continue;
                    }
                    let fields = c.1.outputs.clone();
                    for field in fields {
                        ui.selectable_value(&mut port_field, field.clone(), field);
                    }
                }
            });
    });
    input_port.input.id = port_id;
    input_port.input.field = port_field;
}

pub fn input_port(
    ui: &mut Ui,
    id_tmp: &mut String,
    id: &mut String,
    id_ports: &Vec<(crate::common::Id, Ports)>,
) {
    ui.horizontal(|ui| {
        let id_label = ui.label("Id: ");
        let r = ui
            .text_edit_singleline(&mut *id_tmp)
            .labelled_by(id_label.id);
        if r.lost_focus() && *id_tmp != *id {
            *id = unique_component_name(id_ports, (*id_tmp).as_str());
        }
    });
}
