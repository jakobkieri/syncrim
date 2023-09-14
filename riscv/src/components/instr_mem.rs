use asm_riscv::{self};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    ops::Range,
    rc::Rc,
};

use log::trace;
use riscv_asm_strings::Stringify;
use serde::{Deserialize, Serialize};
use syncrim::common::{Component, Condition, Input, OutputType, Ports, Simulator};

#[derive(Serialize, Deserialize)]
pub struct InstrMem {
    pub id: String,
    pub pos: (f32, f32),
    pub bytes: BTreeMap<usize, u8>,
    pub pc: Input,
    pub range: Range<usize>,
    pub breakpoints: Rc<RefCell<HashSet<usize>>>,
    pub symbols: HashMap<usize, String>,
    pub le: bool,
}

#[typetag::serde()]
impl Component for InstrMem {
    fn to_(&self) {
        println!("InstrMem");
    }
    fn get_id_ports(&self) -> (String, Ports) {
        (
            self.id.clone(),
            Ports {
                inputs: vec![self.pc.clone()],
                out_type: OutputType::Combinatorial,
                outputs: vec!["instruction".into()],
            },
        )
    }

    fn clock(&self, simulator: &mut Simulator) -> Result<(), Condition> {
        // get instr at pc/4
        let pc: u32 = simulator.get_input_value(&self.pc).try_into().unwrap();
        let instr = if self.le{ (*self.bytes.get(&((pc) as usize)).unwrap() as u32) << 24
            | (*self.bytes.get(&((pc + 1) as usize)).unwrap() as u32) << 16
            | (*self.bytes.get(&((pc + 2) as usize)).unwrap() as u32) << 8
            | (*self.bytes.get(&((pc + 3) as usize)).unwrap() as u32)}
        else{(*self.bytes.get(&((pc) as usize)).unwrap() as u32)
            | (*self.bytes.get(&((pc + 1) as usize)).unwrap() as u32) << 8
            | (*self.bytes.get(&((pc + 2) as usize)).unwrap() as u32) << 16
            | (*self.bytes.get(&((pc + 3) as usize)).unwrap() as u32) << 24};
        //the asm_riscv crate incorrectly panics when trying from instead of
        //returning Err, catch it and handle instead
        let instruction_fmt = {
            format!(
                "{:?}",
                match asm_riscv::I::try_from(instr) {
                    Ok(i) => i.to_string(),
                    Err(_) => "Unknown instruction".to_string(),
                }
            )
        }; 
        trace!("instruction: {}", instruction_fmt);
        trace!("pc:0x{:08x}", pc);
        // set output
        simulator.set_out_value(&self.id, "instruction", instr);
        if !self.breakpoints.borrow().contains(&(pc as usize)) {
            Ok(())
        } else {
            Err(Condition::Halt(format!("Breakpoint at {}", pc)))
        }
    }
}
mod test {
    #![allow(unused_imports)]
    use super::*;

    use std::rc::Rc;
    use syncrim::{
        common::{ComponentStore, Input, Simulator},
        components::ProbeOut,
    };
    #[test]
    fn test_inst_mem() {
        let mut instr_mem = BTreeMap::new();
        for i in 0u32..6u32 {
            let bytes = i.to_be_bytes();
            instr_mem.insert((i * 4) as usize, bytes[0]);
            instr_mem.insert((i * 4 + 1) as usize, bytes[1]);
            instr_mem.insert((i * 4 + 2) as usize, bytes[2]);
            instr_mem.insert((i * 4 + 3) as usize, bytes[3]);
        }
        let cs = ComponentStore {
            store: vec![
                Rc::new(ProbeOut::new("pc")),
                Rc::new(InstrMem {
                    id: "imem".to_string(),
                    pos: (0.0, 0.0),
                    pc: Input::new("pc", "out"),
                    bytes: instr_mem,
                    range: Range {
                        start: 0,
                        end: 0x1000,
                    },
                    breakpoints: Rc::new(RefCell::new(HashSet::new())),
                    symbols: HashMap::new(),
                    le: true,
                }),
            ],
        };

        let mut simulator = Simulator::new(&cs);
        assert_eq!(simulator.cycle, 1);

        // outputs
        let imem_out = &Input::new("imem", "instruction");
        for i in 0..6 {
            simulator.set_out_value("pc", "out", i * 4);
            simulator.clock();
            assert_eq!(simulator.get_input_value(imem_out), i.into());
        }
    }
}
