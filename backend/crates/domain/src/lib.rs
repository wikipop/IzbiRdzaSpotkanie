//! Domain types shared by the gateway and the nano services.
//!
//! The crate has no HTTP or storage dependencies — it is pure logic,
//! fully covered by unit tests.

mod bitmap;
mod id;
mod nick;
mod slot;
mod state;

pub use bitmap::{BitmapError, SlotBitmap};
pub use id::{EventId, IdError, UserId};
pub use nick::{Nick, NickError};
pub use slot::{Day, SlotError, SlotId};
pub use state::{State, StateError};
