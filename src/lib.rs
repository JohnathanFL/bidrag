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
use std::ops::Deref;


#[derive(Debug, Copy, Clone, NewType)]
pub struct Axis(usize);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum CtrlType {
    GPAxis, GPButton, MouseAxis, MouseButton, Keyboard
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum MouseAxis {
    X, Y,
    // TODO: Wheel
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Control {
    ctrlType: CtrlType, // What kind of device? (Note: Separates axes and buttons)
    // TODO: Maybe make the determinant 0 be an "update all"?
    determinant: u64,   // Which device (Gamepad 1,2, etc)
    ctrlID: u64,        // Which part of that device?
}

impl Control {
    pub fn new(determinant: u64, ctrlType: CtrlType, ctrlID: u64) -> Control {
        return Control {
            determinant, ctrlType, ctrlID
        };
    }
}


#[derive(Debug, Clone)]
pub struct InputSubsystem {
    /// For computing relative mouse positions.
    windowSize: (f64, f64),
    /// Mouse input will be done on a relative-to-last basis.
    prevMousePos: (f64, f64),
    /// Mouse sensitivity for X and Y axes, respectively.
    mouseSens: (f64, f64),
    /// Indexes into this->axes.
    axisNames: HashMap<String, Axis>,
    /// Which control updates which axes.
    axisBindings: BTreeMap<Control, Vec<Axis>>,
    /// The current and previous value of each axis, respectively.
    axes: Vec<[f64; 2]>
}

impl InputSubsystem {
    pub fn new(mouseSens: (f64, f64), windowSize: (f64, f64)) -> InputSubsystem {
        return InputSubsystem{
            windowSize,
            prevMousePos: (0.0, 0.0),
            mouseSens,
            axisNames: HashMap::new(),
            axisBindings: BTreeMap::new(),
            axes: Vec::new()
        };
    }

    pub fn update_bindings(&mut self, control: Control, newVal: f64) {
        for index in self.axisBindings.get(&control) {
            self.axes[**index][1] = self.axes[**index][0];
            self.axes[**index][0] = newVal;
        }
    }


    pub fn add_binding(&mut self, name: String, boundTo: Control)-> Axis {
        let index = Axis(self.axes.len()); // TODO: Able to remove bindings. Will break this line.

        self.axes.push([0.0; 2]);
        self.axisNames.insert(name, index);

        if let Some(vec) = self.axisBindings.get_mut(&boundTo) {
            vec.push(index);
        }

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