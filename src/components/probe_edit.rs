#[cfg(feature = "gui-egui")]
use crate::common::EguiComponent;
use crate::common::{Component, Condition, Id, OutputType, Ports, Signal, Simulator};
use log::*;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

pub const PROBE_EDIT_OUT_ID: &str = "out";

#[derive(Serialize, Deserialize, Clone)]
pub struct ProbeEdit {
    pub(crate) id: Id,
    pub(crate) pos: (f32, f32),
    pub(crate) edit_history: Arc<RwLock<Vec<TextSignal>>>, // will contain the next editable value
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextSignal {
    pub text: String,
    pub signal: Signal,
}

#[typetag::serde]
impl Component for ProbeEdit {
    fn to_(&self) {
        trace!("ProbeEdit");
    }
    #[cfg(feature = "gui-egui")]
    fn dummy(&self, id: &str, pos: (f32, f32)) -> Box<Rc<dyn EguiComponent>> {
        Box::new(Rc::new(ProbeEdit::new(id, (pos.0, pos.1))))
    }
    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                // Constants do not take any inputs
                vec![],
                OutputType::Combinatorial,
                // Single output value
                vec![PROBE_EDIT_OUT_ID],
            ),
        )
    }

    // propagate editable value
    fn clock(&self, simulator: &mut Simulator) -> Result<(), Condition> {
        let mut history = self.edit_history.write().unwrap();
        trace!("{} history {:?}", self.id, history);
        let current = history.last().unwrap().clone();
        // set output to current value
        simulator.set_out_value(&self.id, "out", current.signal.get_value());
        // push to prepare data for next;
        history.push(current);
        Ok(())
    }

    // reverse simulation, notice does not touch simulator state, its just internal
    fn un_clock(&self, _: &Simulator) {
        let mut edit_history = self.edit_history.write().unwrap();
        trace!("{} history {:?}", self.id, edit_history);
        let _next = edit_history.pop().unwrap(); // pop the next editable value
        let _current = edit_history.pop().unwrap(); // pop current editable value
        let prev = edit_history.pop().unwrap(); // pop the prev editable value
        trace!("next {:?}", _next);
        trace!("current {:?}", _current);
        trace!("prev {:?}", prev);
        edit_history.push(prev.clone()); // push as current
        edit_history.push(prev); // push as next (to be edited)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ProbeEdit {
    pub fn new(id: &str, pos: (f32, f32)) -> Self {
        ProbeEdit {
            id: id.into(),
            pos,
            // initiate internal history
            edit_history: Arc::new(RwLock::new(vec![TextSignal {
                text: "0".to_string(),
                signal: 0.into(),
            }])),
        }
    }

    pub fn rc_new(id: &str, pos: (f32, f32)) -> Rc<Self> {
        Rc::new(ProbeEdit::new(id, pos))
    }
}
