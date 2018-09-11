#![feature(const_vec_new)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

// TODO: Removable binds
// TODO: Joysticks
// TODO: Investigate multi-KB/M support. Could be done in here at least by adding a u8 to all
// // bindings (Which I will have to anyway for joysticks)
// TODO: Feature dependant things to auto convert from glfw/sdl2/etc types
// TODO: Binding keys to functions

#[macro_use]
extern crate newtype;

use std::collections::{HashMap, BTreeMap};
use std::ops::Range;


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, NewType)]
#[repr(transparent)]
pub struct Axis(usize);


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ControlType {
    Keyboard, MouseAxis, MouseButton, GamepadButton, GamepadAxis
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Control {
    ty: ControlType,
    discriminant: usize, // I.E which gamepad or (if ever implemented) which keyboard
    control: usize
}

impl Control {
    pub fn new(ty: ControlType, discriminant: usize, control: usize) -> Control {
        Control {
            ty, discriminant, control
        }
    }
}



const NUM_MOUSE_BUTTONS: usize = 8;
const MAX_GAMEPADS: u8 = 16; // Good default. Is also max for GLFW
const MAX_GP_BUTTONS: u8 =  32; // Seems like a logical max. Humans aren't octopi
const MAX_GP_AXES: u8 = 32;

/// Each Vec<usize> is mapping to indicies in InputSubsystem->axes
#[derive(Debug, Clone)]
struct BindingTree(BTreeMap<Control, Vec<Axis>>);

impl BindingTree {
    fn new() -> BindingTree {
        BindingTree(BTreeMap::new())
    }

    /// Get indices of all axes with this binding
    fn get_indices(&self, control: &Control) -> &Vec<Axis> {
        static emptyVec: Vec<Axis> = Vec::new();

        let tree = &self.0;

        return tree.get(control).unwrap_or(&emptyVec);
    }

    fn add_binding(&mut self, binding: &Control, index: Axis) {
        let tree = &mut self.0;

        if let Some(indices) = tree.get_mut(binding) {
            indices.push(index);
            return;
        }

        tree.insert(*binding, vec![index]);

    }
}

#[derive(Debug, Clone)]
pub struct InputSubsystem {
    windowSize: (f64, f64),

    /// Mouse input will be done on a relative-to-last basis
    prevMousePos: (f64, f64),
    mouseSens: (f64, f64),
    /// Indexes into this->axes
    axisNames: HashMap<String, Axis>,
    /// Which thing updates each axis. MUST be kept parallel to this->axes
    axisBindings: BindingTree,
    /// The current and previous value of each axis, respectively
    axes: Vec<[f64; 2]>
}

impl InputSubsystem {
    pub fn new(mouseSens: (f64, f64), windowSize: (f64, f64)) -> InputSubsystem {
        InputSubsystem{
            windowSize,
            prevMousePos: (0.0, 0.0),
            mouseSens,
            axisNames: HashMap::new(),
            axisBindings: BindingTree::new(),
            axes: Vec::new()
        }

    }

    pub fn update_bindings(&mut self, control: &Control, newVal: f64) {
        let indices = self.axisBindings.get_indices(control);

        for index in indices {
            self.axes[**index][1] = self.axes[**index][0];
            self.axes[**index][0] = newVal;
        }
    }


    pub fn add_binding(&mut self, name: String, boundTo: &Control)-> Axis {
        let index = Axis(self.axes.len()); //TODO: Able to remove bindings. Will break this
        // line.
        self.axes.push([0.0; 2]);

        self.axisNames.insert(name, index);

        self.axisBindings.add_binding(boundTo, index);


        return index;
    }

    /// ALWAYS cache this. Although it's probably not too expensive, avoid calling as often as
    /// possible. normal get() is O(1), this is likely worse due to HashMap
    pub fn get_index(&self, name: &String) -> Axis {
        return self.axisNames[name];
    }

    #[inline]
    pub fn get(&self, index: usize) -> f64 {
        return self.axes[index][0];
    }

    #[inline]
    pub fn get_prev(&self, index: usize) -> f64 {
        return self.axes[index][1];
    }

    #[inline]
    pub fn get_delta(&self, index: usize) -> f64 {
        return self.get(index) - self.get_prev(index);
    }

    #[inline]
    pub fn get_down(&self, index: usize, threshhold: Option<f64>) -> bool {
        return self.get(index) > threshhold.unwrap_or(0.9);
    }
}