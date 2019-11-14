//! Raw message codes for each message type.
#![allow(missing_docs)]

pub const HELLO: u64 = 1;
pub const WELCOME: u64 = 2;
pub const ABORT: u64 = 3;
pub const CHALLENGE: u64 = 4;
pub const AUTHENTICATE: u64 = 5;
pub const GOODBYE: u64 = 6;
pub const ERROR: u64 = 8;
pub const PUBLISH: u64 = 16;
pub const PUBLISHED: u64 = 17;
pub const SUBSCRIBE: u64 = 32;
pub const SUBSCRIBED: u64 = 33;
pub const UNSUBSCRIBE: u64 = 34;
pub const UNSUBSCRIBED: u64 = 35;
pub const EVENT: u64 = 36;
pub const CALL: u64 = 48;
pub const CANCEL: u64 = 49;
pub const RESULT: u64 = 50;
pub const REGISTER: u64 = 64;
pub const REGISTERED: u64 = 65;
pub const UNREGISTER: u64 = 66;
pub const UNREGISTERED: u64 = 67;
pub const INVOCATION: u64 = 68;
pub const INTERRUPT: u64 = 69;
pub const YIELD: u64 = 70;
