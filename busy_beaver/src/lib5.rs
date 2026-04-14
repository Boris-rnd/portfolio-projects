use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct TuringMachine {
    current_state: u8,
    pos: u8,
    mem: TuringMem,
}

#[derive(Debug, Clone)]
pub struct TuringMem {
    mem: [u32; 8], // 4 × 64 = 256 bits
}

impl TuringMem {
    const ELE_COUNT: usize = 8;
    /// Size of elements in bits !
    const ELE_SIZE: usize = (std::mem::size_of::<Self>() * 8 / Self::ELE_COUNT);
    pub fn new() -> Self {
        Self {
            mem: std::array::from_fn(|_| 0),
        }
    }

    pub fn read(&self, pos: u8) -> bool {
        let i = pos as usize;
        let (block, bit) = (i / Self::ELE_SIZE, i % Self::ELE_SIZE);
        (self.mem[block] >> bit) & 1 != 0
    }

    pub fn write(&mut self, pos: u8, value: bool) {
        let i = pos as usize;
        let (block, bit) = (i / Self::ELE_SIZE, i % Self::ELE_SIZE);
        if value {
            self.mem[block] |= 1 << bit;
        } else {
            self.mem[block] &= !(1 << bit);
        }
    }
}
impl Default for TuringMem {
    fn default() -> Self {
        Self {
            mem: std::array::from_fn(|_| 0),
        }
    }
}

static mut STATES: std::sync::OnceLock<Vec<State>> = std::sync::OnceLock::new();
const EMPTY_STATE: State = State::with_actions(0);

pub fn busy_beaver(n: usize, found_max: usize) -> usize {
    let mut states = Vec::with_capacity(256 * n * n);
    for state0 in 0..n {
        for state1 in 0..n {
            for actions_pair in 0..=255 {
                states.push(State::with_actions(actions_pair))
            }
        }
    }
    let _ = unsafe { STATES.take() };
    unsafe { STATES.set(states).unwrap() };

    let max = Arc::new(Mutex::new(0));
    let mut states = Vec::with_capacity(n);
    states.push(&EMPTY_STATE);
    let mut threads = vec![];
    for state0 in states_iter() {
        *states.last_mut().unwrap() = state0;

        let local_states = states.clone(); // clone for the thread
        let local_max = Arc::clone(&max); // share max safely

        threads.push(std::thread::spawn(move || {
            let curr = busy_beaver_recurse(&local_states, n, found_max);
            {
                let mut m = local_max.lock().unwrap();
                if curr > *m {
                    *m = curr
                }
            }
        }));
    }
    for t in threads {
        t.join().unwrap()
    }
    // max = busy_beaver_recurse(&states, n, found_max);
    *max.clone().lock().unwrap()
}
// Assumes ACTIONS is set
fn states_iter() -> impl Iterator<Item = &'static State> {
    unsafe { STATES.get().unwrap().into_iter() }
}

fn busy_beaver_recurse(states: &[&State], n_states: usize, found_max: usize) -> usize {
    debug_assert!(states.len() <= n_states);
    let mut states = states.to_vec();
    states.push(&EMPTY_STATE); // Gets overwritten
    let mut max = 0;
    for last_state in states_iter() {
        // actions.into_iter().flat_map(|i| actions.into_iter().map(move |j| (i.clone(), j.clone())))
        *states.last_mut().unwrap() = last_state;
        let curr_max = if n_states == states.len() {
            let mem: TuringMem = Default::default();
            let mut machine = TuringMachine {
                current_state: 0,
                pos: (found_max + 1) as u8,
                mem,
            };
            if busy_beaver_filter(&machine, &states, found_max) {
                continue;
            }
            execute_turing_machine(&mut machine, &states, found_max).unwrap_or(0)
        } else {
            busy_beaver_recurse(&states, n_states, found_max)
        };
        if curr_max > max {
            // print!("Curr max: {curr_max}\r");
            max = curr_max
        }
    }
    max
}

fn busy_beaver_filter(machine: &TuringMachine, states: &[&State], found_max: usize) -> bool {
    // if states[0].halt_0() {
    //     return true
    // }

    false
}

fn execute_turing_machine(
    machine: &mut TuringMachine,
    states: &[&State],
    max_steps: usize,
) -> Option<usize> {
    let mut steps = 0;
    let mut state = states[machine.current_state as usize];
    loop {
        if steps > max_steps {
            return None;
        } // Machine not stopping but halting problem + busy beaver already done =)
        steps += 1;
        match machine.mem.read(machine.pos) {
            false => {
                if state.hlt0 {
                    break;
                }
                machine.mem.write(machine.pos, state.write0);
                if state.mov0 {
                    machine.pos += 1;
                } else {
                    machine.pos = machine.pos.checked_add_signed(-1).unwrap();
                }
                if state.is_change_state0 {
                    state = states[state.change_state0]
                }
            }, //(state.hlt0, state.mov0, state.write0, (state.is_change_state0, state.change_state0)),
            true => {
                if state.hlt1 {
                    break;
                }
                machine.mem.write(machine.pos, state.write1);
                if state.mov1 {
                    machine.pos += 1;
                } else {
                    machine.pos = machine.pos.checked_add_signed(-1).unwrap();
                }
                if state.is_change_state1 {
                    state = states[state.change_state1]
                }
            }, //(state.hlt1, state.mov1, state.write1, (state.is_change_state1, state.change_state1)),
        };

    }
    Some(steps)
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct State {
    /// Bitfield:
    /// 1<<0 => mov Left/Right on 0
    /// 1<<1 => mov Left/Right on 1
    /// 1<<2 => write 0/1 on 0
    /// 1<<3 => write 0/1 on 1
    /// 1<<4 => halt on 0
    /// 1<<5 => halt on 1
    /// 1<<6 => change state on 0
    /// 1<<7 => change state on 1
    // actions: u8,
    pub mov0: bool,
    pub mov1: bool,
    pub write0: bool,
    pub write1: bool,
    pub hlt0: bool,
    pub hlt1: bool,
    pub is_change_state0: bool,
    pub is_change_state1: bool,
    pub change_state0: usize,
    pub change_state1: usize,
}
impl State {
    const fn with_actions(actions_pair: u8) -> Self {
        Self {
            mov0: actions_pair & (1<<0) != 0,
            mov1:   actions_pair & (1<<1) != 0,
            write0: actions_pair & (1<<2) != 0,
            write1: actions_pair & (1<<3) != 0,
            hlt0:   actions_pair & (1<<4) != 0,
            hlt1:   actions_pair & (1<<5) != 0,
            is_change_state0: actions_pair & (1<<6) != 0,
            is_change_state1: actions_pair & (1<<7) != 0,
            change_state0: 0,
            change_state1: 0,
        }
    }
    // fn mov_0(&self) -> bool {
    //     // self.mov_0
    //     // (self.actions & (1 << 0)) != 0
    // }
    // fn mov_1(&self) -> bool {
    //     // self.mov_1
    //     // (self.actions & (1 << 1)) != 0
    // }
    // fn write_0(&self) -> bool {
    //     // self.write_0
    //     // (self.actions & (1 << 2)) != 0
    // }
    // fn write_1(&self) -> bool {
    //     // self.write_1
    //     // (self.actions & (1 << 3)) != 0
    // }
    // fn halt_0(&self) -> bool {
    //     // self.halt_0
    //     // (self.actions & (1 << 4)) != 0
    // }
    // fn halt_1(&self) -> bool {
    //     // self.halt_1
    //     // (self.actions & (1 << 5)) != 0
    // }
    // fn change_state_0(&self) -> bool {
    //     // self.change_state_0
    //     // (self.actions & (1 << 6)) != 0
    // }
    // fn change_state_1(&self) -> bool {
    //     // self.change_state_1
    //     // (self.actions & (1 << 7)) != 0
    // }
}
// impl Actions {
//     pub fn apply(&self, machine: &mut TuringMachine) {
//         machine.mem.write(machine.pos, self.write);
//         machine.pos = machine.pos.checked_add_signed(self.mov).unwrap();

//         machine.current_state = self.change_state
//     }
// }

// Loop detector
//

// pub struct MachineIterator<'a> {
//     curr: (usize, usize),
//     actions: &'a Vec<Actions>,
// }
// impl<'a> Iterator for StatesIterator<'a> {
//     type Item = State;

//     fn next(&mut self) -> Option<Self::Item> {
//         let action0 = self.actions[self.curr.0].clone();
//         let action1 = self.actions[self.curr.1].clone();
//         self.curr.1 += 1;
//         if self.curr.1 == self.actions.len() {
//             self.curr.1 = 0;
//             self.curr.0 += 1;
//             if self.curr.0 == self.actions.len() {
//                 return None;
//             }
//         }
//         // if action0.halt {
//         //     self.curr.1 += 1;

//         //     if self.curr.1 == self.actions.len() {
//         //         self.curr.1 = 0;
//         //         self.curr.0 += 1;
//         //         if self.curr.0 == self.actions.len() {
//         //             return None
//         //         }
//         //     }
//         // }
//         Some((action0, action1))
//     }
// }
