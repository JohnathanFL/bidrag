// TODO: Joysticks
// TODO: Investigate multi-KB/M support. Could be done in here at least by adding a u8 to all
// // bindings (Which I will have to anyway for joysticks)
// TODO: Feature dependant things to auto convert from glfw/sdl2/etc types

use std::collections::{HashMap, BTreeMap};
use std::sync::mpsc::Receiver;
use std::ops::Range;

/// Since it's used by both SDL2 and GLFW
pub type Key = i32;
pub type MouseButton = u8;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseAxis {
    X = 0,Y = 1
}

#[derive(Debug, Copy, Clone)]
pub enum Binding {
    KBKey(Key), MAxis(MouseAxis), MButton(MouseButton)
}



const NUM_MOUSE_BUTTONS: usize = 8;
/// Each Vec<usize> is mapping to indicies in InputSubsystem->axes
/// As far as I can tell, this structure SHOULD lead to faster access times by binding, although
/// it's now a pain in the GLFW to look up binding by axis.
#[derive(Debug, Clone)]
struct BindingTree {
    keyBindings: BTreeMap<Key, Vec<usize>>, // TODO: Seems like this one could be optimized
    mouseButtonBindings: [Vec<usize>; NUM_MOUSE_BUTTONS],
    mouseAxisBindings: [Vec<usize>; 2]
}

impl BindingTree {
    fn new() -> BindingTree {
        BindingTree {
            keyBindings: BTreeMap::new(),
            // Forgive me rust gods, for I have sinned... greatly.
            // I blame the lack of collect-to-array
            mouseButtonBindings: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(),
                Vec::new(), Vec::new(), Vec::new()],
            mouseAxisBindings: [Vec::new(), Vec::new()]
        }
    }

    fn get_indicies(&self, typeOf: Binding) -> Option<&Vec<usize>> {
        /// Even if we can't find a key for it, the show must go on!
        match typeOf {
            Binding::KBKey(key) => return self.keyBindings.get(&key),
            Binding::MAxis(axis) => return Some(&self.mouseAxisBindings[axis as usize]),
            Binding::MButton(btn) => return Some(&self.mouseButtonBindings[btn as usize]),
            _ => panic!("NOT YET IMPLEMENTED!")
        }
    }

    fn add_binding(&mut self, binding: Binding, index: usize) {
        match binding {
            Binding::KBKey(key) => {
                if !self.keyBindings.contains_key(&key) {
                    self.keyBindings.insert(key.clone(), vec![index]);
                } else {
                    self.keyBindings.get_mut(&key).unwrap().push(index);
                }
            },
            Binding::MAxis(axis) => {
                self.mouseAxisBindings[axis as usize].push(index);
            },
            Binding::MButton(btn) => {
                self.mouseButtonBindings[btn as usize].push(index);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputSubsystem {
    /// Mouse input will be done on a relative-to-last basis
    prevMousePos: (f64, f64),
    mouseSens: (f64, f64),
    /// Indexes into this->axes
    axisNames: HashMap<String, usize>,
    /// Which thing updates each axis. MUST be kept parallel to this->axes
    axisBindings: BindingTree,
    /// The current and previous value of each axis, respectively
    axes: Vec<[f32; 2]>
}

// Makes the code a little cleaner and (I think) is technically faster. (Although maybe not after
// optimizations.)
// TODO: Profile this
const PRESSED_LOOKUP: [f32; 2] = [0.0, 1.0];

impl InputSubsystem {
    pub fn new(mouseSens: (f64, f64)) -> InputSubsystem {
        InputSubsystem{
            prevMousePos: (0.0, 0.0),
            mouseSens,
            axisNames: HashMap::new(),
            axisBindings: BindingTree::new(),
            axes: Vec::new()
        }

    }

    //
    // Originally the updates were unified, but I found that that created far more complexity
    // than it was worth.
    //

    /// Update all axes which depend on this key
    pub fn update_kb_bind(&mut self, key: Key, pressed: bool) {
        if let Some(bindings) = self.axisBindings.get_indicies(Binding::KBKey(key)) {
            for index in bindings {
                self.axes[*index][1] = self.axes[*index][0];
                self.axes[*index][0] = PRESSED_LOOKUP[pressed as usize];
            }
        }
    }

    /// Update all axes which depend on the mouse's position
    pub fn update_mouseaxes_bind(&mut self, axes: (f64, f64)) {
        if let Some(bindings) = self.axisBindings.get_indicies(Binding::MAxis(MouseAxis::X)) {
            let newVal = (axes.0 - self.prevMousePos.0) * self.mouseSens.1;
            for index in bindings {
                self.axes[*index][1] = self.axes[*index][0];
                self.axes[*index][0] = newVal  as f32;
            }
        }
        if let Some(bindings) = self.axisBindings.get_indicies(Binding::MAxis(MouseAxis::Y)) {
            let newVal = (axes.1 - self.prevMousePos.1) * self.mouseSens.1;
            for index in bindings {
                self.axes[*index][1] = self.axes[*index][0];
                self.axes[*index][0] = newVal as f32;
            }
        }

        self.prevMousePos = axes;
    }

    /// Update all axes bound to the mouse button
    pub fn update_mousebutton_bind(&mut self, btn: MouseButton, pressed: bool) {
        if let Some(bindings) = self.axisBindings.get_indicies(Binding::MButton(btn)) {
            for index in bindings {
                self.axes[*index][1] = self.axes[*index][0];
                self.axes[*index][0] = PRESSED_LOOKUP[pressed as usize];
            }
        }
    }


    pub fn add_binding(&mut self, name: String, boundTo: Binding)-> usize {
        let index = self.axes.len(); //TODO: Able to remove bindings. Will break this line.
        self.axes.push([0.0; 2]);

        self.axisNames.insert(name, index);

        self.axisBindings.add_binding(boundTo, index);


        return index;
    }

    /// ALWAYS cache this. Although it's probably not too expensive, avoid calling as often as
    /// possible. normal get() is O(1), this is likely worse due to HashMap
    pub fn get_index(&self, name: &String) -> usize {
        return self.axisNames[name];
    }

    pub fn get(&self, index: usize) -> f32 {
        return self.axes[index][0];
    }
    pub fn get_prev(&self, index: usize) -> f32 {
        return self.axes[index][1];
    }
    pub fn get_delta(&self, index: usize) -> f32 {
        return self.get(index) - self.get_prev(index);
    }
}