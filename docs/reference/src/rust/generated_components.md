# Generated components

Exported component from the macro or the main file that inherit `Window` or `Dialog` is mapped to a Rust structure.

The components are generated and re-exported to the location of the [`include_modules!`] or [`slint!`] macro.
It is represented as a struct with the same name as the component.

For example, if you have

```slint,no-preview
export component MyComponent inherits Window { /*...*/ }
```

in the .slint file, it will create a
```rust
struct MyComponent{ /*...*/ }
```

A component is instantiated using the `fn new() -> Self` function. The following
convenience functions are available through the [`ComponentHandle`] implementation:

  - `fn clone_strong(&self) -> Self`: creates a strongly referenced clone of the component instance.
  - `fn as_weak(&self) -> Weak`: to create a [weak](Weak) reference to the component instance.
  - `fn show(&self)`: to show the window of the component.
  - `fn hide(&self)`: to hide the window of the component.
  - `fn run(&self)`: a convenience function that first calls `show()`,
    followed by spinning the event loop, and `hide()` when returning from the event loop.
  - `fn global<T: Global<Self>>(&self) -> T`: an accessor to the global singletons,

For each top-level property
  - A setter `fn set_<property_name>(&self, value: <PropertyType>)`
  - A getter `fn get_<property_name>(&self) -> <PropertyType>`

For each top-level callback
  - `fn invoke_<callback_name>(&self)`: to invoke the callback
  - `fn on_<callback_name>(&self, callback: impl Fn(<CallbackArgs>) + 'static)`: to set the callback handler.

Note: All dashes (`-`) are replaced by underscores (`_`) in names of types or functions.

After instantiating the component, call [`ComponentHandle::run()`] on show it on the screen and spin the event loop to
react to input events. To show multiple components simultaneously, call [`ComponentHandle::show()`] on each instance.
Call [`run_event_loop()`] when you're ready to enter the event loop.

The generated component struct acts as a handle holding a strong reference (similar to an `Rc`). The `Clone` trait is
not implemented. Instead you need to make explicit [`ComponentHandle::clone_strong`] and [`ComponentHandle::as_weak`]
calls. A strong reference should not be captured by the closures given to a callback, as this would produce a reference
loop and leak the component. Instead, the callback function should capture a weak component and update that to a strong
reference when needed.
