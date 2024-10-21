# Exported Global singletons

When you export a [global singleton](https://slint.dev/releases/", env!("CARGO_PKG_VERSION"), "/docs/slint/src/language/syntax/globals.html) from the main file,
it is also generated with the exported name. Like the main component, the generated struct have
inherent method to access the properties and callback:

For each property
  - A setter: `fn set_<property_name>(&self, value: <PropertyType>)`
  - A getter: `fn get_<property_name>(&self) -> <PropertyType>`

For each callback
  - `fn invoke_<callback_name>(&self, <CallbackArgs>) -> <ReturnValue>` to invoke the callback
  - `fn on_<callback_name>(&self, callback: impl Fn(<CallbackArgs>) + 'static)` to set the callback handler.

The global can be accessed with the [`ComponentHandle::global()`] function, or with [`Global::get()`]

See the [documentation of the `Global` trait](Global) for an example.

**Note**: Global singletons are instantiated once per component. When declaring multiple components for `export` to Rust,
each instance will have their own instance of associated globals singletons.
ub mod docs;
