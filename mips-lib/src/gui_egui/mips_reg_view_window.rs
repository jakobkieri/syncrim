use egui::{RichText, ScrollArea, TextWrapMode, Ui, ViewportBuilder, ViewportId};
use std::collections::{HashMap, HashSet};

//use crate::components::{MemOpSize, MipsMem};
use MIPS_disassembly;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct RegViewWindow {
    pub visible: bool,
    title: String,
    id: String,
    row_offset: u32,
    max_rows: u32,

    // used for formatting the view
    big_endian: bool,
    format: DataFormat,

    // used to determine if section, symbols and other markers should be shown
    show_settings: ShowSettings,

    // used for show register
    register_values: Option<[u32; 32]>,
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
enum DataFormat {
    Hex,
    HexAndMips,
    Bin,
    DecSigned,
    DecUnsigned,
    Byte,
    ByteAndUtf8,
}

#[derive(Clone, Serialize, Deserialize)]
struct ShowSettings {
    symbols: bool,
    sections: bool,
    program_counter: bool,
    registers: [bool; 32],
}

const REG_NAMES: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
    "s7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "fp",
    "ra",
];

impl RegViewWindow {
    // creates a new memory view window with id string and the given memory
    pub fn new(id: String, title: String) -> Self {
        RegViewWindow {
            title,
            id,
            visible: false,
            row_offset: 0,
            max_rows: 1024,
            big_endian: true, // big endian is default on mips
            format: DataFormat::Hex,
            show_settings: ShowSettings {
                symbols: true,
                sections: false,
                program_counter: false,
                registers: [false; 32],
            },
            register_values: None,
        }
    }

    /*
    /// This sets the format to hex + mips and if possible goes to the section .text
    pub fn set_code_view(mut self, mem: Option<&MipsMem>) -> MemViewWindow {
        // find if value ".text" exists, if so go to that
        if let Some(m) = mem {
            match m.get_section_table().iter().find_map(|(adrs, name)| {
                if name == ".text" {
                    Some(adrs)
                } else {
                    None
                }
            }) {
                Some(adrs) => self.go_to_address = GoAddress::Top(*adrs),
                None => self.go_to_address = GoAddress::None,
            };
        }

        // set
        self.format = DataFormat::HexAndMips;
        self.show_settings.registers[31] = true;
        // add PC_IM extra symbol and set to visible
        // Decided to use PC_IM, for consistence with the pipeline model
        self.dynamic_symbols.insert("PC_IM".into(), (0, true));
        self
    }

    /// This sets the format to byte + utf8 and if possible goes to the section .data
    pub fn set_data_view(mut self, mem: Option<&MipsMem>) -> MemViewWindow {
        if let Some(m) = mem {
            // find if value ".text" exists
            match m.get_section_table().iter().find_map(|(adrs, name)| {
                if name == ".data" {
                    Some(adrs)
                } else {
                    None
                }
            }) {
                Some(adrs) => self.go_to_address = GoAddress::Top(*adrs),
                None => self.go_to_address = GoAddress::Top(0x1000),
            };
        }
        self.format = DataFormat::ByteAndUtf8;
        self
    }*/

    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        };

        ctx.show_viewport_immediate(
            ViewportId::from_hash_of(&self.id),
            ViewportBuilder::default().with_title(&self.title),
            |ctx, _class| {
                // If window is close is sent set visible to false
                // WARNING, DON'T USE CONTEXT INSIDE READER: WILL CAUSE DEADLOCK
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.visible = false
                }

                // Render top panel with go to, format and show menus
                self.render_top(ctx);

                egui::CentralPanel::default().show(ctx, |ui| {
                    let h = ui.text_style_height(&egui::TextStyle::Body);
                    /*
                    // if self.go_to_address is none this functions does nothing but return the passed scrollArea
                    //   +2 for the show more buttons
                    scr_area.show_rows(ui, h, (self.max_rows + 2) as usize, |ui, draw_range| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                        ui.set_width(ui.available_width());
                        for i in draw_range.clone() {
                            self.render_scroll_area_item(ui, i);
                        }
                    });*/
                })
            },
        );
    }

    fn render_top(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top(self.id.clone()).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Go to", |ui| {
                    ui.separator();

                    let mut close_menu = false;
                    ui.menu_button("section", |ui| {
                        // for (key, v) in sections {
                        //     if ui.button(format!("{} {:#0x}", v, key)).clicked() {
                        //         self.go_to_address = set_address(&self.go_type, *key);
                        //         ui.close_menu();
                        //         close_menu = true;
                        //     }
                        // }
                    });

                    ui.separator();
                    ui.menu_button("Format", |ui| {
                        ui.selectable_value(&mut self.big_endian, false, "Little Endian");
                        ui.selectable_value(&mut self.big_endian, true, "Big Endian");
                        ui.separator();
                        ui.selectable_value(&mut self.format, DataFormat::Hex, "Hex");
                        ui.selectable_value(&mut self.format, DataFormat::HexAndMips, "Hex + mips");
                        ui.selectable_value(
                            &mut self.format,
                            DataFormat::DecSigned,
                            "Decimal Singed",
                        );
                        ui.selectable_value(
                            &mut self.format,
                            DataFormat::DecUnsigned,
                            "Decimal Unsigned",
                        );
                        ui.selectable_value(&mut self.format, DataFormat::Bin, "Binary");
                        ui.selectable_value(&mut self.format, DataFormat::Byte, "Bytes");
                        ui.selectable_value(
                            &mut self.format,
                            DataFormat::ByteAndUtf8,
                            "Bytes + UTF8",
                        );
                    });
                    ui.menu_button("Show", |ui| {
                        ui.checkbox(&mut self.show_settings.symbols, "Symbols");
                        ui.checkbox(&mut self.show_settings.sections, "Sections");
                        if self.register_values.is_some() {
                            ui.separator();

                            ui.checkbox(&mut self.show_settings.registers[28], "Global Pointer");
                            ui.checkbox(&mut self.show_settings.registers[29], "Stack Pointer");
                            ui.checkbox(&mut self.show_settings.registers[30], "Frame Pointer");
                            ui.checkbox(&mut self.show_settings.registers[31], "Return address");
                            ui.separator();
                            ui.menu_button("Other register", |ui| {
                                ScrollArea::vertical().show(ui, |ui| {
                                    for (i, name) in REG_NAMES.iter().enumerate() {
                                        ui.checkbox(
                                            &mut self.show_settings.registers[i],
                                            format!("${}", name),
                                        );
                                    }
                                });
                            });
                        }
                    });
                });
            });
        });
        /*
        /// NOTE borrows mem
        fn render_scroll_area_item(&mut self, ui: &mut Ui, scroll_area_row: usize) {
            let more_row_text = RichText::new(format!("show {} more rows", &self.max_rows / 2));
            if scroll_area_row == 0 {
                if self.row_offset == 0 {
                    _ = ui.small_button(more_row_text.clone().strikethrough());
                } else if ui.small_button(more_row_text).clicked() {
                    // 4* to get memory address
                    // -1 because the button takes up a row
                };
            } else {
                // -4 is to allow for space for the show more button
                let address = scroll_area_row as u32 * 4 + self.row_offset * 4 - 4;
                if ui
                    .label(
                        RichText::new(format!(
                            "{}{:#010x}\t {:015} {}",
                            address,
                            self.format_row(address, mem),
                            match self.get_symbols_etc_at_address(&address, mem) {
                                Some(string) => format!("\t<= {}", string),
                                None => String::new(),
                            }
                        ))
                        .monospace(),
                    )
                    .clicked()
                {};
            }
        }

        // TODO symbol or sect might not be word aligned,
        // since we check word aligned addresses we might miss the symbol/reg ect
        fn get_symbols_etc_at_address(&self, adrs: &u32) -> Option<String> {
            let mut out_vec: Vec<&str> = vec![];

            for (name, _) in self
                .dynamic_symbols
                .iter()
                .filter(|(_name, (sym_adrs, vis))| sym_adrs == adrs && *vis)
            {
                out_vec.push(name)
            }
            if self.show_settings.sections && sect.contains_key(adrs) {
                out_vec.push(sect.get(adrs).unwrap())
            }
            if self.show_settings.symbols && sym.contains_key(adrs) {
                out_vec.push(sym.get(adrs).unwrap())
            }

            if let Some(reg) = &self.register_values {
                for (i, show) in self.show_settings.registers.iter().enumerate() {
                    if *show && (reg[i] & !0b11) == *adrs {
                        out_vec.push(REG_NAMES[i])
                    }
                }
            }

            if out_vec.is_empty() {
                None
            } else {
                Some(out_vec.join(", "))
            }
        }*/
    }
}
