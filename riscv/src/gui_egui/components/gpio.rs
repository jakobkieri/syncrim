use crate::components::GPIO;
use egui::{Color32, CornerRadius, Pos2, Rect, Response, Shape, Stroke, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use syncrim::common::{EguiComponent, Ports, Simulator};
use syncrim::gui_egui::component_ui::{
    drag_logic, input_change_id, input_selector, pos_drag_value, properties_window,
    rect_with_hover, visualize_ports,
};
use syncrim::gui_egui::editor::{EditorMode, EditorRenderReturn, GridOptions};
use syncrim::gui_egui::gui::EguiExtra;
use syncrim::gui_egui::helper::offset_helper;

#[typetag::serde]
impl EguiComponent for GPIO {
    fn render(
        &self,
        ui: &mut Ui,
        _context: &mut EguiExtra,
        _simulator: Option<&mut Simulator>,
        offset: Vec2,
        scale: f32,
        clip_rect: Rect,
        editor_mode: EditorMode,
    ) -> Option<Vec<Response>> {
        // 21x41
        // middle: 11x 21y (0 0)
        let oh: fn((f32, f32), f32, Vec2) -> Pos2 = offset_helper;
        let offset_old = offset;
        let mut offset = offset;
        offset.x += self.pos.0 * scale;
        offset.y += self.pos.1 * scale;
        let s = scale;
        let o = offset;

        // The shape
        let rect = Rect {
            min: oh((-self.width / 2f32, -self.height / 2f32), s, o),
            max: oh((self.width / 2f32, self.height / 2f32), s, o),
        };
        ui.painter().add(Shape::rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke {
                width: scale,
                color: Color32::BLACK,
            },
            egui::StrokeKind::Inside,
        ));

        let r = rect_with_hover(rect, clip_rect, editor_mode, ui, self.id.clone(), |ui| {
            ui.label(format!("Id: {}", self.id.clone()));
            ui.label("GPIO");
        });
        match editor_mode {
            EditorMode::Simulator => {
                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.set_clip_rect(rect);
                    ui.push_id(1337, |ui| {
                        TableBuilder::new(ui)
                            .column(Column::initial(30.0))
                            .column(Column::initial(15.0))
                            .column(Column::initial(15.0))
                            .column(Column::initial(15.0))
                            .column(Column::initial(15.0))
                            .column(Column::initial(15.0))
                            .column(Column::initial(15.0))
                            .column(Column::initial(15.0))
                            .column(Column::initial(15.0))
                            .header(10.0, |mut header| {
                                header.col(|ui| {
                                    ui.heading("Pin");
                                });
                                header.col(|ui| {
                                    ui.heading("7");
                                });
                                header.col(|ui| {
                                    ui.heading("6");
                                });
                                header.col(|ui| {
                                    ui.heading("5");
                                });
                                header.col(|ui| {
                                    ui.heading("4");
                                });
                                header.col(|ui| {
                                    ui.heading("3");
                                });

                                header.col(|ui| {
                                    ui.heading("2");
                                });

                                header.col(|ui| {
                                    ui.heading("1");
                                });

                                header.col(|ui| {
                                    ui.heading("0");
                                });
                            })
                            .body(|mut body| {
                                body.row(15.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(format!("State"));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(7).unwrap().state as u32
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(6).unwrap().state as u32
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(5).unwrap().state as u32
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(4).unwrap().state as u32
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(3).unwrap().state as u32
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(2).unwrap().state as u32
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(1).unwrap().state as u32
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            self.pins.0.borrow().get(0).unwrap().state as u32
                                        ));
                                    });
                                })
                            });
                    });
                });
            }
            _ => visualize_ports(ui, self.ports_location(), offset_old, scale, clip_rect),
        }
        Some(vec![r])
    }

    fn render_editor(
        &mut self,
        ui: &mut Ui,
        context: &mut EguiExtra,
        simulator: Option<&mut Simulator>,
        offset: Vec2,
        scale: f32,
        clip_rect: Rect,
        id_ports: &[(syncrim::common::Id, Ports)],
        grid: &GridOptions,
        editor_mode: EditorMode,
    ) -> EditorRenderReturn {
        let r_vec = GPIO::render(
            self,
            ui,
            context,
            simulator,
            offset,
            scale,
            clip_rect,
            editor_mode,
        )
        .unwrap();
        let resp = &r_vec[0];
        let delete = drag_logic(
            ui.ctx(),
            resp,
            &mut self.pos,
            &mut context.pos_tmp,
            scale,
            offset,
            grid,
        );

        properties_window(
            ui,
            self.id.clone(),
            resp,
            &mut context.properties_window,
            |ui| {
                let mut clicked_dropdown = false;
                input_change_id(ui, &mut context.id_tmp, &mut self.id, id_ports);
                pos_drag_value(ui, &mut self.pos);
                clicked_dropdown |= input_selector(
                    ui,
                    &mut self.data_i,
                    crate::components::GPIO_DATA_I_ID.to_string(),
                    id_ports,
                    self.id.clone(),
                );
                clicked_dropdown |= input_selector(
                    ui,
                    &mut self.addr_i,
                    crate::components::GPIO_ADDR_I_ID.to_string(),
                    id_ports,
                    self.id.clone(),
                );
                clicked_dropdown |= input_selector(
                    ui,
                    &mut self.we_i,
                    crate::components::GPIO_WE_I_ID.to_string(),
                    id_ports,
                    self.id.clone(),
                );
                clicked_dropdown |= input_selector(
                    ui,
                    &mut self.size_i,
                    crate::components::GPIO_SIZE_I_ID.to_string(),
                    id_ports,
                    self.id.clone(),
                );
                clicked_dropdown |= input_selector(
                    ui,
                    &mut self.se_i,
                    crate::components::GPIO_SE_I_ID.to_string(),
                    id_ports,
                    self.id.clone(),
                );
                clicked_dropdown
            },
        );

        EditorRenderReturn {
            delete,
            resp: Some(r_vec),
        }
    }

    fn ports_location(&self) -> Vec<(syncrim::common::Id, Pos2)> {
        let own_pos = Vec2::new(self.pos.0, self.pos.1);
        vec![
            (
                crate::components::GPIO_DATA_I_ID.to_string(),
                Pos2::new(-self.width / 5f32, -self.height / 2f32) + own_pos,
            ),
            (
                crate::components::GPIO_ADDR_I_ID.to_string(),
                Pos2::new(self.width / 5f32, -self.height / 2f32) + own_pos,
            ),
            (
                crate::components::GPIO_SIZE_I_ID.to_string(),
                Pos2::new(2f32 * (self.width / 5f32), -self.height / 2f32) + own_pos,
            ),
            (
                crate::components::GPIO_WE_I_ID.to_string(),
                Pos2::new(2f32 * (-self.width / 5f32), -self.height / 2f32) + own_pos,
            ),
            (
                crate::components::GPIO_SE_I_ID.to_string(),
                Pos2::new(0.0, -self.height / 2f32) + own_pos,
            ),
        ]
    }

    fn top_padding(&self) -> f32 {
        self.height / 4f32
    }

    fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }

    fn get_pos(&self) -> (f32, f32) {
        self.pos
    }
}
