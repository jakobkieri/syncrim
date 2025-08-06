use egui::{RichText, ScrollArea, TextWrapMode, Ui, ViewportBuilder, ViewportId};
use std::collections::{HashMap, HashSet};

use crate::components::{MemOpSize, MipsMem};
use MIPS_disassembly;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct MemViewWindow {
    pub visible: bool,
    title: String,
    id: String,
    row_offset: u32,
    max_rows: u32,
    /// when set to top, the given address will be displayed at the top of the scroll area
    /// center, the center of the scroll area
    /// bottom, the bottom of the scroll area
    #[serde(skip)]
    go_to_address: GoAddress,
    // determents what is used as GoAddress in the top bar
    #[serde(skip, default = "MemViewWindow::go_type_def")]
    go_type: GoAddress,
    // used when user wants to go to another address
    #[serde(skip)]
    custom_address: u32,

    // used for formatting the view
    big_endian: bool,
    format: DataFormat,

    // used to determine if section, symbols and other markers should be shown
    show_settings: ShowSettings,

    // used for show register
    register_values: Option<[u32; 32]>,

    // used to show pc and jump to pc
    // why not a Rc<InstrMem>? because that would cause circular dependency and a memory leak
    dynamic_symbols: HashMap<String, (u32, bool)>,

    // Added when user clicks a row, and removed when clicked again
    break_points: HashSet<u32>,
}

#[derive(PartialEq, Clone, Default)]
enum GoAddress {
    Top(u32),
    Center(u32),
    Bottom(u32),
    #[default]
    None,
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

fn set_address(adrs: &GoAddress, new_adrs: u32) -> GoAddress {
    match adrs {
        GoAddress::Top(_) => GoAddress::Top(new_adrs),
        GoAddress::Center(_) => GoAddress::Center(new_adrs),
        GoAddress::Bottom(_) => GoAddress::Bottom(new_adrs),
        GoAddress::None => GoAddress::None,
    }
}

impl MemViewWindow {
    fn go_type_def() -> GoAddress {
        GoAddress::Top(0)
    }
    // creates a new memory view window with id string and the given memory
    pub fn new(id: String, title: String) -> Self {
        MemViewWindow {
            title,
            id,
            visible: false,
            row_offset: 0,
            max_rows: 1024,
            go_to_address: GoAddress::None,
            go_type: GoAddress::Top(0),
            custom_address: 0,
            big_endian: true, // big endian is default on mips
            format: DataFormat::Hex,
            show_settings: ShowSettings {
                symbols: true,
                sections: false,
                program_counter: false,
                registers: [false; 32],
            },
            register_values: None,
            dynamic_symbols: HashMap::new(),
            break_points: HashSet::new(),
        }
    }

    /// set register values, allows to display where they point as well as jump to them
    pub fn set_reg_values(&mut self, reg_values: [u32; 32]) {
        self.register_values = Some(reg_values);
    }
    /// Set the extra symbols address, if no symbol exist add that symbol
    pub fn set_dynamic_symbol(&mut self, symbol: &str, adrs: u32) {
        match self.dynamic_symbols.get_mut(symbol) {
            Some((cur_adrs, _vis)) => {
                *cur_adrs = adrs;
            }
            None => {
                self.dynamic_symbols
                    .insert(symbol.to_string(), (adrs, false));
            }
        }
    }
    // replaces all dynamic symbols with the given new_dynamic_symbols
    pub fn set_all_dynamic_symbols(&mut self, new_dynamic_symbols: HashMap<String, (u32, bool)>) {
        for (name, (adress, bool)) in new_dynamic_symbols {
            self.set_dynamic_symbol(name.as_str(), adress);
        }
    }
    /// Get the address of a symbol, if no such symbol exist return None
    pub fn get_dynamic_symbol(&self, symbol: &str) -> Option<u32> {
        self.dynamic_symbols.get(symbol).map(|(adrs, _)| *adrs)
    }

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
    }

    pub fn is_break_point(&self, address: &u32) -> bool {
        self.break_points.contains(address)
    }

    pub fn render(&mut self, ctx: &egui::Context, mem: &MipsMem) {
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
                self.render_top(ctx, mem);

                egui::CentralPanel::default().show(ctx, |ui| {
                    let h = ui.text_style_height(&egui::TextStyle::Body);

                    // if self.go_to_address is none this functions does nothing but return the passed scrollArea
                    let scr_area = self.scroll_to_address(ui, ScrollArea::vertical());
                    //   +2 for the show more buttons
                    scr_area.show_rows(ui, h, (self.max_rows + 2) as usize, |ui, draw_range| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                        ui.set_width(ui.available_width());
                        for i in draw_range.clone() {
                            self.render_scroll_area_item(ui, i, mem);
                        }
                    });
                })
            },
        );
    }

    fn render_top(&mut self, ctx: &egui::Context, mem: &MipsMem) {
        egui::TopBottomPanel::top(self.id.clone()).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Go to", |ui| {
                    // used to allow the user to select if the address should be show in the top, center, bottom of the scroll area
                    ui.menu_button("show address at", |ui| {
                        ui.selectable_value(&mut self.go_type, GoAddress::Top(0), "top");
                        ui.selectable_value(&mut self.go_type, GoAddress::Center(0), "center");
                        ui.selectable_value(&mut self.go_type, GoAddress::Bottom(0), "bottom");
                    });
                    ui.separator();

                    let mut close_menu = false;

                    // add submenu with a button for each symbol, which sets self.go_to_address
                    ui.menu_button("symbol", |ui| {
                        let because_lifetimes_sad = mem.get_symbol_table();
                        let mut sections = because_lifetimes_sad.iter().collect::<Vec<_>>();
                        sections.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

                        ScrollArea::vertical().show(ui, |ui| {
                            let because_lifetimes_sad = mem.get_symbol_table();
                            let mut symbols = because_lifetimes_sad.iter().collect::<Vec<_>>();
                            symbols.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

                            for (key, v) in symbols {
                                if ui.button(format!("{} {:#0x}", v, key)).clicked() {
                                    self.go_to_address = set_address(&self.go_type, *key);
                                    ui.close_menu();
                                    close_menu = true;
                                }
                            }
                        });
                    });
                    ui.menu_button("section", |ui| {
                        let because_lifetimes_sad = mem.get_section_table();
                        let mut sections = because_lifetimes_sad.iter().collect::<Vec<_>>();
                        sections.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

                        // for (key, v) in sections {
                        //     if ui.button(format!("{} {:#0x}", v, key)).clicked() {
                        //         self.go_to_address = set_address(&self.go_type, *key);
                        //         ui.close_menu();
                        //         close_menu = true;
                        //     }
                        // }
                        ScrollArea::vertical().show(ui, |ui| {
                            let because_lifetimes_sad = mem.get_section_table();
                            let mut section = because_lifetimes_sad.iter().collect::<Vec<_>>();
                            section.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

                            for (key, v) in section {
                                if ui.button(format!("{} {:#0x}", v, key)).clicked() {
                                    self.go_to_address = set_address(&self.go_type, *key);
                                    ui.close_menu();
                                    close_menu = true;
                                }
                            }
                        });
                    });

                    // Does any PC pointer exists, make them visible in this menu for quick access
                    if self.dynamic_symbols.contains_key("PC_IM")
                        || self.dynamic_symbols.contains_key("PC_DE")
                        || self.dynamic_symbols.contains_key("PC_EX")
                        || self.dynamic_symbols.contains_key("PC_DM")
                    {
                        ui.separator();

                        if let Some((adrs, _)) = self.dynamic_symbols.get("PC_IM") {
                            if ui.button(format!("PC_IM ({:#0x})", adrs)).clicked() {
                                self.go_to_address = set_address(&self.go_type, *adrs);
                                ui.close_menu();
                                close_menu = true;
                            }
                        }

                        if let Some((adrs, _)) = self.dynamic_symbols.get("PC_DE") {
                            if ui.button(format!("PC_DE ({:#0x})", adrs)).clicked() {
                                self.go_to_address = set_address(&self.go_type, *adrs);
                                ui.close_menu();
                                close_menu = true;
                            }
                        }

                        if let Some((adrs, _)) = self.dynamic_symbols.get("PC_EX") {
                            if ui.button(format!("PC_EX ({:#0x})", adrs)).clicked() {
                                self.go_to_address = set_address(&self.go_type, *adrs);
                                ui.close_menu();
                                close_menu = true;
                            }
                        }

                        if let Some((adrs, _)) = self.dynamic_symbols.get("PC_DM") {
                            if ui.button(format!("PC_DM ({:#0x})", adrs)).clicked() {
                                self.go_to_address = set_address(&self.go_type, *adrs);
                                ui.close_menu();
                                close_menu = true;
                            }
                        }
                    }

                    if let Some(reg) = &self.register_values {
                        ui.separator();

                        let gp = reg[28];
                        if ui.button(format!("Global pointer ({:#0x})", gp)).clicked() {
                            self.go_to_address = set_address(&self.go_type, gp)
                        }
                        let sp = reg[29];
                        if ui.button(format!("Stack pointer ({:#0x})", sp)).clicked() {
                            self.go_to_address = set_address(&self.go_type, sp)
                        }
                        let fp = reg[30];
                        if ui.button(format!("Frame pointer ({:#0x})", fp)).clicked() {
                            self.go_to_address = set_address(&self.go_type, fp)
                        }
                        let ra = reg[31];
                        if ui.button(format!("Return address ({:#0x})", gp)).clicked() {
                            self.go_to_address = set_address(&self.go_type, ra)
                        }

                        ui.separator();

                        ui.menu_button("Other Register", |ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                for (i, name) in REG_NAMES.iter().enumerate() {
                                    let val = reg[i];
                                    if ui.button(format!("${} {:#0x}", name, val)).clicked() {
                                        self.go_to_address = set_address(&self.go_type, val);
                                        ui.close_menu();
                                        close_menu = true;
                                    }
                                }
                            })
                        });
                    }

                    ui.separator();
                    ui.menu_button("Dynamic symbols", |ui| {
                        let mut extra_symbols = self.dynamic_symbols.iter().collect::<Vec<_>>();
                        extra_symbols.sort_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap());
                        for (symbol, (adrs, _)) in extra_symbols {
                            if ui.button(format!("{} {:#0x}", symbol, adrs)).clicked() {
                                self.go_to_address = set_address(&self.go_type, *adrs);
                                ui.close_menu();
                                close_menu = true;
                            }
                        }
                    });

                    ui.separator();
                    ui.menu_button("Other Address", |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.custom_address)
                                .hexadecimal(8, true, true)
                                .prefix("0x"),
                        );
                        if ui.button("Go").clicked() {
                            self.go_to_address = set_address(&self.go_type, self.custom_address);
                            close_menu = true;
                        }
                    });

                    if close_menu {
                        ui.close_menu();
                    }
                });
                ui.menu_button("Format", |ui| {
                    ui.selectable_value(&mut self.big_endian, false, "Little Endian");
                    ui.selectable_value(&mut self.big_endian, true, "Big Endian");
                    ui.separator();
                    ui.selectable_value(&mut self.format, DataFormat::Hex, "Hex");
                    ui.selectable_value(&mut self.format, DataFormat::HexAndMips, "Hex + mips");
                    ui.selectable_value(&mut self.format, DataFormat::DecSigned, "Decimal Singed");
                    ui.selectable_value(
                        &mut self.format,
                        DataFormat::DecUnsigned,
                        "Decimal Unsigned",
                    );
                    ui.selectable_value(&mut self.format, DataFormat::Bin, "Binary");
                    ui.selectable_value(&mut self.format, DataFormat::Byte, "Bytes");
                    ui.selectable_value(&mut self.format, DataFormat::ByteAndUtf8, "Bytes + UTF8");
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
                    if !self.dynamic_symbols.is_empty() {
                        ui.separator();
                        ui.menu_button("Dynamic symbols", |ui| {
                            for (sym, (_, vis)) in self.dynamic_symbols.iter_mut() {
                                ui.checkbox(vis, sym);
                            }
                        });
                    }
                });
                ui.menu_button("Break points", |ui| {
                    if ui.button("Clear all breakpoints").clicked() {
                        self.break_points.clear();
                        ui.close_menu();
                    }
                });
            });
        });
    }
    /// NOTE borrows mem
    fn render_scroll_area_item(&mut self, ui: &mut Ui, scroll_area_row: usize, mem: &MipsMem) {
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
    }
    /// NOTE BORROWS MEM
    fn format_row(&self, adrs: u32, mem: &MipsMem) -> String {
        let data_u32 = mem.get_unaligned(adrs, MemOpSize::Word, false, self.big_endian);
        let bytes = mem
            .get_unaligned(adrs, MemOpSize::Word, false, true)
            .to_be_bytes();
        // TODO get symbol table clones the hashmap, this is infective
        let sym_tab = mem.get_symbol_table();
        match self.format {
            DataFormat::Hex => {
                format!("{:#010x}", data_u32)
            }
            DataFormat::HexAndMips => {
                format!(
                    "{:#010x}   {:015}",
                    data_u32,
                    MIPS_disassembly::get_disassembly_adv(
                        data_u32,
                        adrs,
                        &sym_tab,
                        &MIPS_disassembly::MipsDisassemblyOptions::new(true, true)
                    )
                )
            }
            DataFormat::Bin => {
                format!("{:032b}", data_u32)
            }
            DataFormat::DecSigned => {
                format!("{}", data_u32 as i32)
            }
            DataFormat::DecUnsigned => {
                format!("{}", data_u32)
            }
            DataFormat::Byte => {
                format!(
                    "{:02x} {:02x} {:02x} {:02x}",
                    bytes[0], bytes[1], bytes[2], bytes[3],
                )
            }
            DataFormat::ByteAndUtf8 => {
                format!(
                    "{:02x} {:02x} {:02x} {:02x} \"{}\"",
                    bytes[0],
                    bytes[1],
                    bytes[2],
                    bytes[3],
                    String::from_utf8_lossy(&bytes).escape_debug()
                )
            }
        }
    }

    /// Scroll the scroll area to the address
    fn scroll_to_address(&mut self, ui: &mut Ui, scroll_area: ScrollArea) -> ScrollArea {
        // if we should not go to an address return
        if self.go_to_address == GoAddress::None {
            return scroll_area;
        }

        let row = match self.go_to_address {
            GoAddress::Top(adrs) => adrs / 4,
            GoAddress::Center(adrs) => adrs / 4,
            GoAddress::Bottom(adrs) => adrs / 4,
            GoAddress::None => panic!("I Should have returned not continue"),
        };

        //make address middle of rows visible rows
        let mut offset = match row.checked_sub(self.max_rows / 2) {
            // don't ask why +1, don't fully know, but has to to with modulo
            Some(v) => v + 1,
            None => 0,
        };

        // align to half max rows
        offset -= offset % (self.max_rows / 2);
        // update offset
        self.row_offset = offset;

        // calculate scroll amount
        let row_height = ui.text_style_height(&egui::TextStyle::Body);
        let y_spacing = ui.style().spacing.item_spacing.y;
        let top = (row - self.row_offset + 1) as f32 * (row_height + y_spacing);

        let scroll = match self.go_to_address {
            GoAddress::Top(_) => top,
            GoAddress::Center(_) => top - ui.available_height() / 2.0,
            GoAddress::Bottom(_) => top - ui.available_height(),
            GoAddress::None => panic!("I Should have returned not continue"),
        };

        self.go_to_address = GoAddress::None;
        scroll_area.vertical_scroll_offset(scroll)
    }

    // TODO symbol or sect might not be word aligned,
    // since we check word aligned addresses we might miss the symbol/reg ect
    fn get_symbols_etc_at_address(&self, adrs: &u32, mem: &MipsMem) -> Option<String> {
        let mut out_vec: Vec<&str> = vec![];
        let sym = mem.get_symbol_table();
        let sect = mem.get_section_table();

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
    }
}
