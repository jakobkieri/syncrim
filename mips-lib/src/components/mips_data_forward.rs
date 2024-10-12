//TODO: add so it can take undefined number of inputs
use log::*;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::rc::Rc;
use syncrim::common::{
    Component, Condition, Id, Input, InputPort, OutputType, Ports, SignalValue, Simulator,
};
pub const DATA_FORWARD_A_IN_ID: &str = "a_in";
pub const DATA_FORWARD_B_IN_ID: &str = "b_in";
pub const DATA_FORWARD_WE_IN_ID: &str = "we_in";

pub const DATA_FORWARD_OUT_ID: &str = "data_forward_out";

#[derive(Serialize, Deserialize, Clone)]
pub struct DataForward {
    pub(crate) id: Id,
    pub(crate) pos: (f32, f32),
    pub(crate) a_in: Input,
    pub(crate) b_in: Input,
    pub(crate) we_in: Input,
}

#[typetag::serde]
impl Component for DataForward {
    fn to_(&self) {
        trace!("AluFroward");
    }

    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![
                    &InputPort {
                        port_id: DATA_FORWARD_A_IN_ID.to_string(),
                        input: self.a_in.clone(),
                    },
                    &InputPort {
                        port_id: DATA_FORWARD_B_IN_ID.to_string(),
                        input: self.b_in.clone(),
                    },
                    &InputPort {
                        port_id: DATA_FORWARD_WE_IN_ID.to_string(),
                        input: self.we_in.clone(),
                    },
                ],
                OutputType::Combinatorial,
                vec![DATA_FORWARD_OUT_ID],
            ),
        )
    }

    // propagate addition to output
    fn clock(&self, simulator: &mut Simulator) -> Result<(), Condition> {
        // get input values
        let a_in: u32 = simulator.get_input_value(&self.a_in).try_into().unwrap();
        let b_in: u32 = simulator.get_input_value(&self.b_in).try_into().unwrap();
        let we_in: u32 = simulator.get_input_value(&self.we_in).try_into().unwrap();

        let result: u32 = if we_in == 1 { (a_in == b_in) as u32 } else { 0 };

        simulator.set_out_value(&self.id, DATA_FORWARD_OUT_ID, SignalValue::Data(result));
        Ok(())
    }

    fn set_id_port(&mut self, target_port_id: Id, new_input: Input) {
        match target_port_id.as_str() {
            DATA_FORWARD_A_IN_ID => self.a_in = new_input,
            DATA_FORWARD_B_IN_ID => self.b_in = new_input,
            _ => (),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DataForward {
    pub fn new(id: &str, pos: (f32, f32), a_in: Input, b_in: Input, we_in: Input) -> Self {
        DataForward {
            id: id.to_string(),
            pos,
            a_in,
            b_in,
            we_in,
        }
    }

    pub fn rc_new(id: &str, pos: (f32, f32), a_in: Input, b_in: Input, we_in: Input) -> Rc<Self> {
        Rc::new(DataForward::new(id, pos, a_in, b_in, we_in))
    }
}
