//! Defines types for Gotham middleware

use std::io;

use handler::HandlerFuture;
use state::State;

pub mod pipeline;
pub mod session;

/// `Middleware` has the opportunity to provide additional behaviour to the `Request` / `Response`
/// interaction. Middleware-specific state data can be recorded in the `State` struct for
/// use elsewhere.
///
/// # Examples
///
/// Taking no action, and immediately passing the `Request` through to the rest of the application:
///
/// ```rust,no_run
/// # extern crate gotham;
/// # extern crate hyper;
/// #
/// # use gotham::handler::HandlerFuture;
/// # use gotham::middleware::Middleware;
/// # use gotham::state::State;
/// #
/// struct NoopMiddleware;
///
/// impl Middleware for NoopMiddleware {
///     fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
///         where Chain: FnOnce(State) -> Box<HandlerFuture> + 'static
///     {
///         chain(state)
///     }
/// }
/// #
/// # fn main() {
/// #  NoopMiddleware {};
/// # }
/// ```
///
/// Recording a piece of state data before passing the request through:
///
/// ```rust,no_run
/// # extern crate gotham;
/// # #[macro_use]
/// # extern crate gotham_derive;
/// # extern crate hyper;
/// #
/// # use gotham::handler::HandlerFuture;
/// # use gotham::middleware::Middleware;
/// # use gotham::state::State;
///
/// #
/// struct MiddlewareWithStateData;
///
/// # #[allow(unused)]
/// # #[derive(StateData)]
/// struct MiddlewareStateData {
///     i: i32,
/// }
///
/// impl Middleware for MiddlewareWithStateData {
///     fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
///         where Chain: FnOnce(State) -> Box<HandlerFuture> + 'static
///     {
///         state.put(MiddlewareStateData { i: 10 });
///         chain(state)
///     }
/// }
/// #
/// # fn main() {
/// #     MiddlewareWithStateData {};
/// # }
/// ```
///
/// Terminating the request early based on some arbitrary condition:
///
/// ```rust,no_run
/// # extern crate gotham;
/// # extern crate hyper;
/// # extern crate futures;
/// #
/// # use gotham::http::response::create_response;
/// # use gotham::handler::HandlerFuture;
/// # use gotham::middleware::Middleware;
/// # use gotham::state::{State, FromState};

/// # use hyper::{Method, StatusCode};
/// # use futures::future;
/// #
/// struct ConditionalMiddleware;
///
/// impl Middleware for ConditionalMiddleware {
///     fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
///         where Chain: FnOnce(State) -> Box<HandlerFuture> + 'static
///     {
///         if *Method::borrow_from(&state) == Method::Get {
///             chain(state)
///         } else {
///             let response = create_response(&state, StatusCode::MethodNotAllowed, None);
///             Box::new(future::ok((state, response)))
///         }
///     }
/// }
/// #
/// # fn main() {
/// #     ConditionalMiddleware {};
/// # }
/// ```
///
/// Asynchronous middleware, which continues the request after some action completes:
///
/// ```rust,no_run
/// # extern crate gotham;
/// # extern crate hyper;
/// # extern crate futures;
/// #
/// # use gotham::handler::HandlerFuture;
/// # use gotham::middleware::Middleware;
/// # use gotham::state::State;

/// # use futures::{future, Future};
/// #
/// struct AsyncMiddleware;
///
/// impl Middleware for AsyncMiddleware {
///     fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
///         where Chain: FnOnce(State) -> Box<HandlerFuture> + 'static
///     {
///         // This could be any asynchronous action. `future::lazy(_)` defers a function
///         // until the next cycle of tokio's event loop.
///         let f = future::lazy(|| future::ok(()));
///         Box::new(f.and_then(move |_| chain(state)))
///     }
/// }
/// #
/// # fn main() {
/// #    AsyncMiddleware {};
/// # }
/// ```
pub trait Middleware {
    /// Entry point to the middleware. To pass the request on to the application, the middleware
    /// invokes the `chain` function with the provided `state` and `request`.
    ///
    /// By convention, the middleware should:
    ///
    /// * Avoid modifying the `Request`, unless it is already determined that the response will be
    ///   generated by the middleware (i.e. without calling `chain`);
    /// * Ensure to pass the same `State` to `chain`, rather than creating a new `State`.
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
        Self: Sized;
}

/// Creates new `Middleware` values.
pub trait NewMiddleware: Sync {
    /// The type of `Middleware` created by the implementor.
    type Instance: Middleware;

    /// Create and return a new `Middleware` value.
    fn new_middleware(&self) -> io::Result<Self::Instance>;
}
