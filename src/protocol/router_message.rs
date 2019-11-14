//! The various types of messages which can be received by the client as part of WAMP. For more
//! details, see [the WAMP protocol specification].
//!
//! [the WAMP protocol specification]: http://wamp-proto.org/spec/

use super::transportable_value::{Dict, List};
use super::{id, msg_code, Id, Uri};

macro_rules! rx_message_type {
    ($name:ident [ $code_name:ident ] { $($fields:tt)* }) => {
        #[derive(Debug, Eq, PartialEq)]
        pub struct $name {
            $($fields)*
        }
        impl RouterMessage for $name {
            const MSG_CODE: u64 = msg_code::$code_name;
        }
    };
}

/// Marker trait for received messages. Do not implement this yourself.
pub trait RouterMessage {
    /// The identifying integer for this message.
    const MSG_CODE: u64;
}

// Session management; used by all types of peers.
rx_message_type!(Welcome [WELCOME] {
    pub session: Id<id::GlobalScope>,
    pub details: Dict,
});

// Session management; used by all types of peers.
rx_message_type!(Abort [ABORT] {
    pub details: Dict,
    pub reason: Uri,
});

// Session management; used by all types of peers.
rx_message_type!(Challenge [CHALLENGE] {
    pub auth_method: String,
    pub extra: Dict,
});

// Session management; used by all types of peers.
rx_message_type!(Goodbye [GOODBYE] {
    pub details: Dict,
    pub reason: Uri,
});

// Message type used by all roles to indicate problems with a request.
rx_message_type!(Error [ERROR] {
    pub request_type: u64,
    pub request: Id<id::SessionScope>,
    pub details: Dict,
    pub error: Uri,
    pub arguments: Option<List>,
    pub arguments_kw: Option<Dict>,
});

// Sent by brokers to subscribers after they are subscribed to a topic.
rx_message_type!(Subscribed [SUBSCRIBED] {
    pub request: Id<id::SessionScope>,
    pub subscription: Id<id::RouterScope>,
});

// Sent by brokers to subscribers after they are unsubscribed from a topic.
rx_message_type!(Unsubscribed [UNSUBSCRIBED] {
    pub request: Id<id::SessionScope>,
});

// Sent by borkers to subscribers to indicate that a message was published to a topic.
rx_message_type!(Event [EVENT] {
    pub subscription: Id<id::RouterScope>,
    pub publication: Id<id::GlobalScope>,
    pub details: Dict,
    pub arguments: Option<List>,
    pub arguments_kw: Option<Dict>,
});

// Sent by brokers to publishers after they publish a message to a topic, if they
// requested acknowledgement.
rx_message_type!(Published [PUBLISHED] {
    pub request: Id<id::SessionScope>,
    pub publication: Id<id::GlobalScope>,
});

// Sent by dealers to callees after an RPC is registered.
rx_message_type!(Registered [REGISTERED] {
    pub request: Id<id::SessionScope>,
    pub registration: Id<id::RouterScope>,
});

// Sent by dealers to callees after an RPC is unregistered.
rx_message_type!(Unregistered [UNREGISTERED] {
    pub request: Id<id::SessionScope>,
});

// Sent by dealers to callees when an RPC they have registered is invoked.
rx_message_type!(Invocation [INVOCATION] {
    pub request: Id<id::SessionScope>,
    pub registration: Id<id::RouterScope>,
    pub details: Dict,
    pub arguments: Option<List>,
    pub arguments_kw: Option<Dict>,
});

// Sent by dealers to callers when an RPC they invoked has completed.
rx_message_type!(Result [RESULT] {
    pub request: Id<id::SessionScope>,
    pub details: Dict,
    pub arguments: Option<List>,
    pub arguments_kw: Option<Dict>,
});
