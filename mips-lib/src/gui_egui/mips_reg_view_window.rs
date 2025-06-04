use egui::{
    debug_text::print, ComboBox, RichText, ScrollArea, TextWrapMode, Ui, ViewportBuilder,
    ViewportId,
};
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
    //format: DataFormat,

    // used to determine if section, symbols and other markers should be shown
    show_settings: ShowSettings,

    // used for show register
    register_values: [u32; 32],
    show_reg_names: bool,
    reg_format: RegFormat,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)] //, Default, PartialEq, PartialOrd, Debug)]
enum RegFormat {
    //#[default]
    Hex,
    Bin,
    DecSigned,
    DecUnsigned,
    UTF8BE,
    UTF8LE,
}

/*#[derive(PartialEq, Clone, Serialize, Deserialize)]
enum DataFormat {
    Hex,
    HexAndMips,
    Bin,
    DecSigned,
    DecUnsigned,
    Byte,
    ByteAndUtf8,
}
*/

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
    /// set register values, allows to display where they point as well as jump to them
    pub fn set_reg_values(&mut self, reg_values: [u32; 32]) {
        self.register_values = reg_values;
    }

    // creates a new memory view window with id string and the given memory
    pub fn new(id: String, title: String) -> Self {
        RegViewWindow {
            title,
            id,
            visible: false,
            row_offset: 0,
            max_rows: 1024,
            big_endian: true, // big endian is default on mips
            //format: DataFormat::Hex,
            show_settings: ShowSettings {
                symbols: true,
                sections: false,
                program_counter: false,
                registers: [false; 32],
            },
            register_values: [0; 32],
            show_reg_names: true,
            reg_format: RegFormat::Hex,
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

                /*
                egui::CentralPanel::default().show(ctx, |ui| {
                    let h = ui.text_style_height(&egui::TextStyle::Body);
                    let scr_area = ScrollArea::vertical();
                    //   +2 for the show more buttons
                    scr_area.show_rows(ui, h, (self.max_rows + 2) as usize, |ui, draw_range| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                        ui.set_width(ui.available_width());
                        for i in draw_range.clone() {
                            //self.render_scroll_area_item(ui, i);
                        }
                    });
                })*/

                self.render_registers(ctx);
            },
        );
    }

    fn render_top(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top(self.id.clone()).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // A toggle button for showing register names
                ui.toggle_value(&mut self.show_reg_names, "Show names");

                // show the display format of the register
                let mut tmp: RegFormat = self.reg_format.clone();
                ComboBox::from_id_source(&self.id)
                    .selected_text(format!("{:?}", tmp))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut tmp, RegFormat::Hex, "Hex");
                        ui.selectable_value(&mut tmp, RegFormat::DecUnsigned, "Decimal");
                        ui.selectable_value(&mut tmp, RegFormat::DecSigned, "Decimal signed");
                        ui.selectable_value(&mut tmp, RegFormat::Bin, "Binary");
                        ui.selectable_value(&mut tmp, RegFormat::UTF8BE, "UTF-8 big endian");
                        ui.selectable_value(&mut tmp, RegFormat::UTF8LE, "UTF-8 little endian");
                    });
                self.reg_format = tmp;
            });
        });

        /*
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
    /*
    fn render_scroll_area_item(&mut self, ui: &mut Ui, scroll_area_row: usize) {
        let more_row_text = RichText::new(format!("show {} more rows", &self.max_rows / 2));
        if scroll_area_row == 0 {
            if self.row_offset == 0 {
                _ = ui.small_button(more_row_text.clone().strikethrough());
            } else if ui.small_button(more_row_text).clicked() {
                // 4* to get memory address
                // -1 because the button takes up a row
                self.go_to_address = GoAddress::Top((self.row_offset - 1) * 4);
            };
        } else if scroll_area_row == self.max_rows as usize + 1 {
            if ui.small_button(more_row_text).clicked() {
                self.go_to_address = GoAddress::Bottom((self.row_offset + self.max_rows) * 4);
            };
        } else {
            // -4 is to allow for space for the show more button
            let address = scroll_area_row as u32 * 4 + self.row_offset * 4 - 4;
            if ui
                .label(
                    RichText::new(format!(
                        "{}{:#010x}\t {:015} {}",
                        match self.break_points.contains(&address) {
                            true => "BREAK ",
                            false => "",
                        },
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
            {
                // was the row clicked if so add breakpoint to address
                match self.break_points.contains(&address) {
                    true => self.break_points.remove(&address),
                    false => self.break_points.insert(address),
                };
            };
        }
    }*/
    // A scroll area with all the registers in one label
    fn render_registers(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                
                for (i, val) in self.register_values.iter().enumerate() {
                    ui.label(
                        RichText::new(format!(
                            "{} \t {}",
                            match self.show_reg_names {
                                true => format!("{:<4}", REG_NAMES[i]),
                                false => format!("r{:<3}", i),
                            }
                            .as_str(),
                            match self.reg_format {
                                RegFormat::Hex => format!("{:#010x}", val),
                                RegFormat::DecSigned => format!("{}", (*val) as i32),
                                RegFormat::DecUnsigned => format!("{}", val),
                                RegFormat::Bin => format!("{:#034b}", val),
                                RegFormat::UTF8BE => String::from_utf8_lossy(&val.to_be_bytes())
                                    .escape_debug()
                                    .to_string(),
                                RegFormat::UTF8LE => String::from_utf8_lossy(&val.to_le_bytes())
                                    .escape_debug()
                                    .to_string(),
                            }
                            .as_str(),
                        ))
                        .monospace(),
                    );
                }
            });
        });
    }
}
