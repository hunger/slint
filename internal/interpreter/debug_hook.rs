// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use std::{collections::HashMap, pin::Pin};

use crate::{eval::ComponentInstance, Value};

use smol_str::SmolStr;

#[derive(Debug, Default)]
struct ValueState {
    values: PropertyValueOverride,
    override_dependency: Option<Pin<Box<i_slint_core::properties::Property<i32>>>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct DebugHookKey {
    id: SmolStr,
    repeat_count: usize,
}

impl DebugHookKey {
    fn new(id: SmolStr, repeat_count: usize) -> Self {
        Self { id, repeat_count }
    }
}

pub type Observer = Box<dyn Fn(&str, usize, &PropertyValueOverride)>;

#[derive(Default)]
struct DebugHookState {
    value_states: HashMap<DebugHookKey, ValueState>,
    observer: Option<Observer>,
}

thread_local! { static DEBUG_HOOK_STATE: std::cell::RefCell<DebugHookState> = Default::default(); }

fn with_debug_hook_mut<F, T>(key: DebugHookKey, func: F) -> T
where
    F: FnOnce(&mut ValueState, i32, &Option<Observer>) -> T,
{
    DEBUG_HOOK_STATE.with(|debug_hook_state| {
        let mut debug_hook_state = debug_hook_state.borrow_mut();

        let observer = debug_hook_state.observer.take();
        let state = debug_hook_state.value_states.entry(key.clone()).or_default();

        if state.override_dependency.is_none() {
            state.override_dependency = Some(Box::pin(i_slint_core::properties::Property::new(0)));
        }

        let override_generation = state.override_dependency.as_ref().unwrap().as_ref().get();

        let result = func(state, override_generation, &observer);
        debug_hook_state.observer = observer;

        result
    })
}

pub(crate) fn set_observer(func: Option<Observer>) {
    DEBUG_HOOK_STATE.with(|debug_hook_state| {
        debug_hook_state.borrow_mut().observer = func;
    })
}

fn find_repeat_count(instance: &ComponentInstance) -> usize {
    // FIXME: Do something to find the component's repeated index.
    match instance {
        ComponentInstance::InstanceRef(_) => 0,
        ComponentInstance::GlobalComponent(_) => 0,
    }
}

pub(crate) fn set_debug_hook_override_value(
    id: SmolStr,
    repeat_count: usize,
    value: Option<Value>,
) -> i32 {
    with_debug_hook_mut(DebugHookKey::new(id, repeat_count), move |state, generation, _| {
        let new_generation = generation.wrapping_add(1);
        state.override_dependency.as_ref().unwrap().as_ref().set(new_generation);
        state.values.override_value = value;

        new_generation
    })
}

/// A struct used to remember and or override properties marked with an `@debug-hook`
/// expression.
#[derive(Clone, Debug, Default)]
pub struct PropertyValueOverride {
    /// The first value ever assigned to ther property
    pub initial_value: Option<Value>,
    /// The value last evaluated by the code wrapped in the @debug-hook
    pub program_value: Option<Value>,
    /// The value the @debug-hook overrides the `program_value` with
    pub override_value: Option<Value>,
}

impl PropertyValueOverride {
    /// Return the value the property should report right now
    ///
    /// This is either the `override_value` (if set) or the `program_value`
    pub fn current_value(&self) -> Option<Value> {
        if self.override_value.is_some() {
            self.override_value.clone()
        } else {
            self.program_value.clone()
        }
    }
}

pub(crate) fn get_debug_hook_property_value_override(
    id: SmolStr,
    repeat_count: usize,
) -> PropertyValueOverride {
    with_debug_hook_mut(DebugHookKey::new(id, repeat_count), move |state, _, _| {
        state.values.clone()
    })
}

pub(crate) fn debug_hook_triggered(
    instance: &ComponentInstance,
    id: SmolStr,
    value: Value,
) -> Value {
    let repeat_count = find_repeat_count(instance);
    with_debug_hook_mut(DebugHookKey::new(id.clone(), repeat_count), move |state, _, observer| {
        if state.values.initial_value.is_none() {
            state.values.initial_value = Some(value.clone());
        }
        state.values.program_value = Some(value);

        if let Some(observer) = observer {
            observer(id.as_str(), repeat_count, &state.values);
        }

        if state.values.override_value.is_some() {
            return state.values.override_value.clone().unwrap();
        }
        state.values.program_value.clone().unwrap()
    })
}
