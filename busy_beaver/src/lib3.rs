#[derive(Debug, Clone)]
pub struct TuringMachine {
    current_state: usize,
    pos: u8,
    mem: TuringMem,
}

// #[derive(Debug, Clone)]
// pub struct TuringMem {
//     mem: [u64; 4], // 4 × 64 = 256 bits
// }

// impl TuringMem {
//     pub fn new() -> Self {
//         Self { mem: [0; 4] }
//     }

//     pub fn read(&self, pos: u8) -> bool {
//         let i = pos as usize;
//         let (block, bit) = (i / 64, i % 64);
//         (self.mem[block] & bit as u64) != 0
//     }

//     pub fn write(&mut self, pos: u8, value: bool) {
//         let i = pos as usize;
//         let (block, bit) = (i / 64, i % 64);
//         if value {
//             self.mem[block] |= 1 << bit;
//         } else {
//             self.mem[block] &= !(1 << bit);
//         }
//     }
// }
// impl Default for TuringMem {
//     fn default() -> Self {
//         Self {
//             mem: std::array::from_fn(|_| 0),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct TuringMem {
    pub mem: [u32; 300],
}
impl TuringMem {
    pub fn read(&self, pos: u8) -> bool {
        self.mem[(pos) as usize] == 1
    }
    pub fn write(&mut self, pos: u8, value: bool) {
        self.mem[(pos) as usize] = if value {1} else {0}
    }
}
impl Default for TuringMem {
    fn default() -> Self {
        Self {
            mem: std::array::from_fn(|_| 0),
        }
    }
}

pub fn busy_beaver(n: usize, found_max: usize) -> usize {
    let action_combinations = [
        Actions {
            mov: -1,
            write: false,
            halt: false,
            change_state: 0,
        }, // change_state is overwritten after
        Actions {
            mov: -1,
            write: false,
            halt: true,
            change_state: 0,
        },
        Actions {
            mov: -1,
            write: true,
            halt: false,
            change_state: 0,
        },
        Actions {
            mov: -1,
            write: true,
            halt: true,
            change_state: 0,
        },
        Actions {
            mov: 1,
            write: false,
            halt: false,
            change_state: 0,
        },
        Actions {
            mov: 1,
            write: false,
            halt: true,
            change_state: 0,
        },
        Actions {
            mov: 1,
            write: true,
            halt: false,
            change_state: 0,
        },
        Actions {
            mov: 1,
            write: true,
            halt: true,
            change_state: 0,
        },
    ]
    .to_vec();
    
    let mut actions = Vec::with_capacity(action_combinations.len() * n);
    for state in 0..n {
        for a in &action_combinations {
            let mut a = a.clone();
            a.change_state = state;
            actions.push(a)
        }
    }

    return busy_beaver_recurse(&actions, &Vec::with_capacity(n), n, found_max);
}

fn busy_beaver_recurse(
    actions: &Vec<Actions>,
    states: &Vec<(Actions, Actions)>,
    n_states: usize,
    found_max: usize,
) -> usize {
    // dbg!(states);
    let iter = StatesIterator {
        curr: (0, 0),
        actions: &actions,
    };
    let mut states = states.clone();
    states.push((Actions::default(), Actions::default())); // Gets overwritten
    let mut max = 0;
    for last_state in iter {
        *states.last_mut().unwrap() = last_state;
        let curr_max = if n_states == states.len() {
            let mem: TuringMem = Default::default();
            let mut machine = TuringMachine {
                current_state: 0,
                pos: (found_max+1) as u8,
                mem,
            };
            execute_turing_machine(&mut machine, &states, found_max).unwrap_or(0)
        } else {
            busy_beaver_recurse(actions, &states, n_states, found_max)
        };
        if curr_max > max {
            // print!("Curr max: {curr_max}\r");
            max = curr_max
        }
    }
    max
}

fn execute_turing_machine(
    machine: &mut TuringMachine,
    states: &Vec<(Actions, Actions)>,
    max_steps: usize,
) -> Option<usize> {
    let mut steps = 0;
    loop {
        if steps > max_steps + 10 {
            return None;
        } // Machine not stopping but halting problem + busy beaver already done =)
        steps += 1;
        let actions = states[machine.current_state];
        match machine.mem.read(machine.pos) {
            false => {
                if actions.0.halt {
                    break;
                }
                actions.0.apply(machine)
            }
            true => {
                if actions.1.halt {
                    break;
                }
                actions.1.apply(machine)
            }
        }
        // if machine.pos > 90 {dbg!(steps, states);}
    }
    Some(steps)
}

pub struct StatesIterator<'a> {
    curr: (usize, usize),
    actions: &'a Vec<Actions>,
}
impl<'a> Iterator for StatesIterator<'a> {
    type Item = (Actions, Actions);

    fn next(&mut self) -> Option<Self::Item> {
        let action0 = self.actions[self.curr.0];
        let action1 = self.actions[self.curr.1];
        self.curr.1 += 1;
        if self.curr.1 == self.actions.len() {
            self.curr.1 = 0;
            self.curr.0 += 1;
            if self.curr.0 == self.actions.len() {
                return None;
            }
        }
        // if action0.halt {
        //     self.curr.1 += 1;

        //     if self.curr.1 == self.actions.len() {
        //         self.curr.1 = 0;
        //         self.curr.0 += 1;
        //         if self.curr.0 == self.actions.len() {
        //             return None
        //         }
        //     }
        // }
        Some((action0, action1))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Actions {
    mov: i8,
    /// false = write 0, true=write 1
    /// To do nothing just write same value
    write: bool,
    halt: bool,
    change_state: usize,
}
impl Actions {
    pub fn apply(&self, machine: &mut TuringMachine) {
        machine.mem.write(machine.pos, self.write);
        machine.pos.checked_add_signed(self.mov).unwrap();
        // match self.write {
        //     true => ,
        //     false => machine.mem.write(machine.pos, 0),
        // };
        // match self.mov {
        //     true => machine.pos += 1,
        //     false => machine.pos -= 1,
        // };
        machine.current_state = self.change_state
    }
}

// Loop detector
//
