use mips_lib::components::*;
use std::path::PathBuf;
#[cfg(feature = "gui-egui")]
use syncrim::gui_egui::editor::Library;
use syncrim::{
    common::{ComponentStore, Input},
    components::*,
    fern::fern_setup,
};

fn main() {
    fern_setup();
    let cs = ComponentStore {
        store: vec![
            ALU::rc_new(
                "alu",
                (200.0, 120.0),
                Input::new("c0", "out"),
                Input::new("c1", "out"),
                Input::new("c2", "out"),
            ),
            ProbeEdit::rc_new("c0", (60.0, 100.0)),
            ProbeEdit::rc_new("c1", (60.0, 140.0)),
            ProbeEdit::rc_new("c2", (60.0, 180.0)),
            Probe::rc_new("p1", (270.0, 120.0), Input::new("alu", ALU_OUT_ID)),
        ],
    };

    let path = PathBuf::from("alu.json");
    cs.save_file(&path);

    #[cfg(feature = "gui-egui")]
    {
        use syncrim::autowire::autowire;
        let cs = autowire(cs);
        cs.save_file(&path);
        syncrim::gui_egui::gui(cs, &path, Library::default()).ok();
    }

    #[cfg(feature = "gui-vizia")]
    syncrim::gui_vizia::gui(cs, &path);
}
