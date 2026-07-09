//! Each command decides what should happen, it delegates
//! how to the `git` module and is something wrong to `checks`.

pub mod check;
pub mod repair;
pub mod save;
pub mod start;
pub mod status;
pub mod sync;
pub mod undo;
pub mod unsync;
