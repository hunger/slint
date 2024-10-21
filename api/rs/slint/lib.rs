// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

// cSpell: ignore buildrs

/*!
# Slint

This crate is the main entry point for embedding user interfaces designed with
[Slint](https://slint.rs/) in Rust programs.

This is the Rust API Refence documentation, see
*/
#![doc = concat!("[Slint Reference Manual](https://slint.dev/releases/", env!("CARGO_PKG_VERSION"), "/docs/slint)")]

/*! for an introduction and Slint language and for integration of the Slint
language into Rust projects.
*/

/*! # Feature flags and backend selection.
Use the following feature flags in your Cargo.toml to enable additional features.
*/
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
/*!
More information about the backend and renderers is available in the
*/
#![doc = concat!("[Slint Documentation](https://slint.dev/releases/", env!("CARGO_PKG_VERSION"), "/docs/slint/src/advanced/backends_and_renderers.html)")]
#![warn(missing_docs)]
#![deny(unsafe_code)]
#![doc(html_logo_url = "https://slint.dev/logo/slint-logo-square-light.svg")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::needless_doctest_main)] // We document how to write a main function

extern crate alloc;

#[cfg(not(feature = "compat-1-2"))]
compile_error!(
    "The feature `compat-1-2` must be enabled to ensure \
    forward compatibility with future version of this crate"
);

pub use slint_macros::slint;

pub use i_slint_core::api::*;
#[doc(hidden)]
#[deprecated(note = "Experimental type was made public by mistake")]
pub use i_slint_core::component_factory::ComponentFactory;
#[cfg(not(target_arch = "wasm32"))]
pub use i_slint_core::graphics::{BorrowedOpenGLTextureBuilder, BorrowedOpenGLTextureOrigin};
// keep in sync with internal/interpreter/api.rs
pub use i_slint_core::graphics::{
    Brush, Color, Image, LoadImageError, Rgb8Pixel, Rgba8Pixel, RgbaColor, SharedPixelBuffer,
};
pub use i_slint_core::model::{
    FilterModel, MapModel, Model, ModelExt, ModelNotify, ModelPeer, ModelRc, ModelTracker,
    ReverseModel, SortModel, StandardListViewItem, TableColumn, VecModel,
};
pub use i_slint_core::sharedvector::SharedVector;
pub use i_slint_core::timers::{Timer, TimerMode};
pub use i_slint_core::{format, string::SharedString};

pub mod private_unstable_api;

/// Enters the main event loop. This is necessary in order to receive
/// events from the windowing system for rendering to the screen
/// and reacting to user input.
/// This function will run until the last window is closed or until
/// [`quit_event_loop()`] is called.
///
/// See also [`run_event_loop_until_quit()`] to keep the event loop running until
/// [`quit_event_loop()`] is called, even if all windows are closed.
pub fn run_event_loop() -> Result<(), PlatformError> {
    i_slint_backend_selector::with_platform(|b| b.run_event_loop())
}

/// Similar to [`run_event_loop()`], but this function enters the main event loop
/// and continues to run even when the last window is closed, until
/// [`quit_event_loop()`] is called.
///
/// This is useful for system tray applications where the application needs to stay alive
/// even if no windows are visible.
pub fn run_event_loop_until_quit() -> Result<(), PlatformError> {
    i_slint_backend_selector::with_platform(|b| {
        #[allow(deprecated)]
        b.set_event_loop_quit_on_last_window_closed(false);
        b.run_event_loop()
    })
}

/// Spawns a [`Future`](core::future::Future) to execute in the Slint event loop.
///
/// This function is intended to be invoked only from the main Slint thread that runs the event loop.
///
/// For spawning a `Send` future from a different thread, this function should be called from a closure
/// passed to [`invoke_from_event_loop()`].
///
/// This function is typically called from a UI callback.
///
/// # Example
///
/// ```rust,no_run
/// slint::spawn_local(async move {
///     // your async code goes here
/// }).unwrap();
/// ```
///
/// # Compatibility with Tokio and other runtimes
///
/// The runtime used to execute the future on the main thread is platform-dependent,
/// for instance, it could be the winit event loop. Therefore, futures that assume a specific runtime
/// may not work. This may be an issue if you call `.await` on a future created by another
/// runtime, or pass the future directly to `spawn_local`.
///
/// Futures from the [smol](https://docs.rs/smol/latest/smol/) runtime always hand off their work to
/// separate I/O threads that run in parallel to the Slint event loop.
///
/// The [Tokio](https://docs.rs/tokio/latest/tokio/index.html) runtime is to the following constraints:
///
/// * Tokio futures require entering the context of a global Tokio runtime.
/// * Tokio futures aren't guaranteed to hand off their work to separate threads and may therefore not complete, because
/// the Slint runtime can't drive the Tokio runtime.
/// * Tokio futures require regular yielding to the Tokio runtime for fairness, a constraint that also can't be met by Slint.
/// * Tokio's [current-thread schedule](https://docs.rs/tokio/latest/tokio/runtime/index.html#current-thread-scheduler)
/// cannot be used in Slint main thread, because Slint cannot yield to it.
///
/// To address these constraints, use [async_compat](https://docs.rs/async-compat/latest/async_compat/index.html)'s [Compat::new()](https://docs.rs/async-compat/latest/async_compat/struct.Compat.html#method.new)
/// to implicitly allocate a shared, multi-threaded Tokio runtime that will be used for Tokio futures.
///
/// The following little example demonstrates the use of Tokio's [`TcpStream`](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) to
/// read from a network socket. The entire future passed to `spawn_local()` is wrapped in `Compat::new()` to make it run:
///
/// ```rust,no_run
/// // A dummy TCP server that once reports "Hello World"
/// # i_slint_backend_testing::init_integration_test_with_mock_time();
/// use std::io::Write;
///
/// let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
/// let local_addr = listener.local_addr().unwrap();
/// let server = std::thread::spawn(move || {
///     let mut stream = listener.incoming().next().unwrap().unwrap();
///     stream.write("Hello World".as_bytes()).unwrap();
/// });
///
/// let slint_future = async move {
///     use tokio::io::AsyncReadExt;
///     let mut stream = tokio::net::TcpStream::connect(local_addr).await.unwrap();
///     let mut data = Vec::new();
///     stream.read_to_end(&mut data).await.unwrap();
///     assert_eq!(data, "Hello World".as_bytes());
///     slint::quit_event_loop().unwrap();
/// };
///
/// // Wrap the future that includes Tokio futures in async_compat's `Compat` to ensure
/// // presence of a Tokio run-time.
/// slint::spawn_local(async_compat::Compat::new(slint_future)).unwrap();
///
/// slint::run_event_loop_until_quit().unwrap();
///
/// server.join().unwrap();
/// ```
///
/// The use of `#[tokio::main]` is **not recommended**. If it's necessary to use though, wrap the call to enter the Slint
/// event loop  in a call to [`tokio::task::block_in_place`](https://docs.rs/tokio/latest/tokio/task/fn.block_in_place.html):
///
/// ```rust, no_run
/// // Wrap the call to run_event_loop to ensure presence of a Tokio run-time.
/// tokio::task::block_in_place(slint::run_event_loop).unwrap();
/// ```
#[cfg(target_has_atomic = "ptr")]
pub fn spawn_local<F: core::future::Future + 'static>(
    fut: F,
) -> Result<JoinHandle<F::Output>, EventLoopError> {
    i_slint_backend_selector::with_global_context(|ctx| ctx.spawn_local(fut))
        .map_err(|_| EventLoopError::NoEventLoopProvider)?
}

/// Include the code generated with the slint-build crate from the build script. After calling `slint_build::compile`
/// in your `build.rs` build script, the use of this macro includes the generated Rust code and makes the exported types
/// available for you to instantiate.
///
/// Check the documentation of the `slint-build` crate for more information.
#[macro_export]
macro_rules! include_modules {
    () => {
        include!(env!("SLINT_INCLUDE_GENERATED"));
    };
}

/// Initialize translations when using the `gettext` feature.
///
/// Call this in your main function with the path where translations are located.
/// This macro internally calls the [`bindtextdomain`](https://man7.org/linux/man-pages/man3/bindtextdomain.3.html) function from gettext.
///
/// The first argument of the macro must be an expression that implements `Into<std::path::PathBuf>`.
/// It specifies the directory in which gettext should search for translations.
///
/// Translations are expected to be found at `<dirname>/<locale>/LC_MESSAGES/<crate>.mo`,
/// where `dirname` is the directory passed as an argument to this macro,
/// `locale` is a locale name (e.g., `en`, `en_GB`, `fr`), and
/// `crate` is the package name obtained from the `CARGO_PKG_NAME` environment variable.
///
/// ### Example
/// ```rust
/// fn main() {
///    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/translations/"));
///    // ...
/// }
/// ```
///
/// For example, assuming this is in a crate called `example` and the default locale
/// is configured to be French, it will load translations at runtime from
/// `/path/to/example/translations/fr/LC_MESSAGES/example.mo`.
///
/// Another example of loading translations relative to the executable:
/// ```rust
/// slint::init_translations!(std::env::current_exe().unwrap().parent().unwrap().join("translations"));
/// ```
#[cfg(feature = "gettext")]
#[macro_export]
macro_rules! init_translations {
    ($dirname:expr) => {
        $crate::private_unstable_api::init_translations(env!("CARGO_PKG_NAME"), $dirname);
    };
}

/// This module contains items that you need to use or implement if you want use Slint in an environment without
/// one of the supplied platform backends such as qt or winit.
///
/// The primary interface is the [`platform::Platform`] trait. Pass your implementation of it to Slint by calling
/// [`platform::set_platform()`] early on in your application, before creating any Slint components.

/// The
#[doc = concat!("[Slint Documentation](https://slint.dev/releases/", env!("CARGO_PKG_VERSION"), "/docs/slint/src/rust/mcu.html)")]
/// has additional examples.

pub mod platform {
    pub use i_slint_core::platform::*;

    /// This module contains the [`femtovg_renderer::FemtoVGRenderer`] and related types.
    ///
    /// It is only enabled when the `renderer-femtovg` Slint feature is enabled.
    #[cfg(all(feature = "renderer-femtovg", not(target_os = "android")))]
    pub mod femtovg_renderer {
        pub use i_slint_renderer_femtovg::FemtoVGRenderer;
        pub use i_slint_renderer_femtovg::OpenGLInterface;
    }
}

#[cfg(any(
    doc,
    all(
        target_os = "android",
        any(feature = "backend-android-activity-05", feature = "backend-android-activity-06")
    )
))]
pub mod android;

/// Helper type that helps checking that the generated code is generated for the right version
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct VersionCheck_1_9_0;

#[cfg(doctest)]
mod compile_fail_tests;
