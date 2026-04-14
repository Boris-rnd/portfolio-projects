use std::sync::{Arc, Mutex, OnceLock};

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
    const ELE_SIZE: usize = (std::mem::size_of::<Self>()*8/Self::ELE_COUNT);
    pub fn new() -> Self {
        Self { mem: std::array::from_fn(|_| 0) }
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
pub const ACTION_COMBINATIONS: [Actions; 8] = [
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
];

static mut ACTIONS: std::sync::OnceLock<Vec<Actions>> = std::sync::OnceLock::new();

pub fn busy_beaver(n: usize, found_max: usize) -> usize {
    let mut actions = Vec::with_capacity(ACTION_COMBINATIONS.len() * n);
    for state in 0..n {
        for a in &ACTION_COMBINATIONS {
            let mut a = a.clone();
            a.change_state = state as u8;
            actions.push(a)
        }
    }
    NON_HALTING_MACHINES.get();
    let _ = unsafe { ACTIONS.take() };
    unsafe { ACTIONS.set(actions).unwrap() };
    let _ = unsafe { NON_HALTING_MACHINES.set(Mutex::new(Vec::new())).unwrap() };
    let max = Arc::new(Mutex::new(0));
    let mut states = Vec::with_capacity(n);
    states.push((Actions::default(), Actions::default()));
    let mut threads = vec![];
    for state0 in states_iter() {
        *states.last_mut().unwrap() = state0;

        let local_states = states.clone(); // clone for the thread
        let local_max = Arc::clone(&max); // share max safely
    
        threads.push(std::thread::spawn(move || {
            let curr = busy_beaver_recurse(&local_states, n, found_max);
            {
                let mut m = local_max.lock().unwrap();
                if curr>*m {*m = curr}
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
fn states_iter() -> StatesIterator<'static> {
    StatesIterator {
        curr: (0, 0),
        actions: unsafe { ACTIONS.get().unwrap() },
    }
}

fn busy_beaver_recurse(
    states: &Vec<(Actions, Actions)>,
    n_states: usize,
    found_max: usize,
) -> usize {
    debug_assert!(states.len()<=n_states);
    let mut states = states.clone();
    states.push((Actions::default(), Actions::default())); // Gets overwritten
    let mut max = 0;
    for last_state in states_iter() { // actions.into_iter().flat_map(|i| actions.into_iter().map(move |j| (i.clone(), j.clone())))
        *states.last_mut().unwrap() = last_state;
        let curr_max = if n_states == states.len() {
            let mem: TuringMem = Default::default();
            let mut machine = TuringMachine {
                current_state: 0,
                pos: (found_max+1) as u8,
                mem,
            };
            if busy_beaver_filter(&machine, &states, found_max) {
                continue
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

fn states_reachable(states: &[(Actions, Actions)]) -> Vec<u8> {
    let mut reachables = vec![0];
    if states[0].0.change_state != 0 || (states[0].1.change_state != 0 && states[0].0.write == true) {
        reachables.push(1);
    }
    reachables
}

fn busy_beaver_filter(machine: &TuringMachine, states: &Vec<(Actions, Actions)>, found_max: usize) -> bool {
    let reachables = states_reachable(states);
    if states[0].0.halt ||
    // If never write anything else than 0, and don't change on 0, then will always be on 0, so skip
    (states[0].0.write == false && states[0].0.change_state == 0) ||
    // If nothing will halt, then it will never halt
    states.iter().all(|st| (reachables.contains(st) && (st.0.halt == true && st.1.halt == true)))
    {
        return true
    }
    false
}

fn execute_turing_machine(
    machine: &mut TuringMachine,
    states: &Vec<(Actions, Actions)>,
    max_steps: usize,
) -> Option<usize> {
    let mut steps = 0;
    loop {
        if steps > max_steps {
            dbg!(states);
            std::process::exit(0);
            return None;
        } // Machine not stopping but halting problem + busy beaver already done =)
        steps += 1;
        let actions = states.get(machine.current_state as usize).unwrap();
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
        let action0 = self.actions[self.curr.0].clone();
        let action1 = self.actions[self.curr.1].clone();
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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Actions {
    mov: i8,
    /// false = write 0, true=write 1
    /// To do nothing just write same value
    write: bool,
    halt: bool,
    change_state: u8,
}
impl Actions {
    pub fn apply(&self, machine: &mut TuringMachine) {
        machine.mem.write(machine.pos, self.write);
        machine.pos = machine.pos.checked_add_signed(self.mov).unwrap();
        
        machine.current_state = self.change_state
    }
}
pub static NON_HALTING_MACHINES: OnceLock<Mutex<Vec<Vec<(Actions, Actions)>>>> = OnceLock::new();

// Loop detector
//
