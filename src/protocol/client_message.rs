use super::transportable_value::{Dict, List};
use super::{id, msg_code, Id, Uri};

/// The various types of messages which can be sent by the client as part of WAMP. For more
/// details, see [the WAMP protocol specification].
///
/// [the WAMP protocol specification]: http://wamp-proto.org/spec/
#[derive(Debug)]
pub enum ClientMessage {
    /// Session management; used by all types of peers.
    Hello {
        /// The realm to connect to.
        realm: Uri,
        /// Describes this peer's capabilities. Could also include a user-agent string.
        details: Dict,
    },

    Authenticate {
        signature: String,
        extra: Dict,
    },

    /// Session management; used by all types of peers.
    Goodbye {
        /// Allows providing optional, additional information.
        details: Dict,
        /// A somewhat known URI describing why the session is being closed.
        reason: Uri,
    },

    /// Message type used by all roles to indicate a problem with a request.
    Error {
        /// The message code for the type of message that caused the error.
        request_type: u64,
        /// The ID of the request that caused the error.
        request: Id<id::SessionScope>,
        /// Optional details describing the error.
        details: Dict,
        /// A somewhat known URI describing the error.
        error: Uri,
        /// A list of positional data.
        arguments: Option<List>,
        /// A dictionary of key-value data.
        arguments_kw: Option<Dict>,
    },

    /// Sent by publishers to brokers when they have a message to send.
    #[cfg(feature = "publisher")]
    Publish {
        /// A request ID.
        request: Id<id::SessionScope>,
        /// Optional additional parameters for the publication.
        options: Dict,
        /// The topic to publish to.
        topic: Uri,
        /// An optional list of positional data.
        arguments: Option<List>,
        /// An optional dictionary of key-value data.
        arguments_kw: Option<Dict>,
    },

    /// Sent by subscribers to brokers when they wish to receive publications sent to a
    /// particular topic.
    #[cfg(feature = "subscriber")]
    Subscribe {
        /// A request ID.
        request: Id<id::SessionScope>,
        /// Optional additional parameters for the requested subscription.
        options: Dict,
        /// The topic to subscribe to.
        topic: Uri,
    },

    /// Sent by subscribers to brokers when they no longer wish to receive publications sent
    /// to a particular topic.
    #[cfg(feature = "subscriber")]
    Unsubscribe {
        /// A request ID.
        request: Id<id::SessionScope>,
        /// The ID of the subscription to cancel.
        subscription: Id<id::RouterScope>,
    },

    /// Sent by callers to dealers when they wish to invoke an RPC.
    #[cfg(feature = "caller")]
    Call {
        /// A request ID.
        request: Id<id::SessionScope>,
        /// Optional additional parameters for the call.
        options: Dict,
        /// The procedure to invoke.
        procedure: Uri,
        /// The positional arguments to the procedure.
        arguments: Option<List>,
        /// The key-value arguments to the procedure.
        arguments_kw: Option<Dict>,
    },

    /// Sent by callees to dealers to register a new RPC.
    #[cfg(feature = "callee")]
    Register {
        /// A request ID.
        request: Id<id::SessionScope>,
        /// Optional additional parameters for the registration.
        options: Dict,
        /// The procedure to register.
        procedure: Uri,
    },

    /// Sent by callees to dealers to remove an existing RPC registration.
    #[cfg(feature = "callee")]
    Unregister {
        /// A request ID.
        request: Id<id::SessionScope>,
        /// The ID of the registration to remove.
        registration: Id<id::RouterScope>,
    },

    /// Sent by callees to dealers to indicate that an RPC is finished.
    #[cfg(feature = "callee")]
    Yield {
        /// The request ID from the invocation.
        request: Id<id::RouterScope>,
        /// Optional additional parameters for the yielded data.
        options: Dict,
        /// Positional returned data.
        arguments: Option<List>,
        /// Key-value returned data.
        arguments_kw: Option<Dict>,
    },
}

impl ClientMessage {
    /// Determines the message code for this message.
    // TODO: UT for this (ugh)
    pub fn get_message_code(&self) -> u64 {
        use ClientMessage::*;

        match self {
            Hello { .. } => msg_code::HELLO,
            Authenticate { .. } => msg_code::AUTHENTICATE,
            Goodbye { .. } => msg_code::GOODBYE,
            Error { .. } => msg_code::ERROR,

            #[cfg(feature = "publisher")]
            Publish { .. } => msg_code::PUBLISH,

            #[cfg(feature = "subscriber")]
            Subscribe { .. } => msg_code::SUBSCRIBE,

            #[cfg(feature = "subscriber")]
            Unsubscribe { .. } => msg_code::UNSUBSCRIBE,

            #[cfg(feature = "caller")]
            Call { .. } => msg_code::CALL,

            #[cfg(feature = "callee")]
            Register { .. } => msg_code::REGISTER,

            #[cfg(feature = "callee")]
            Unregister { .. } => msg_code::UNREGISTER,

            #[cfg(feature = "callee")]
            Yield { .. } => msg_code::YIELD,
        }
    }

    /// Converts this message to a JSON representation.
    // TODO: UT for this (ugh)
    #[cfg(feature = "serde_json")]
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;

        use ClientMessage::*;

        let code = self.get_message_code();
        match self {
            Hello {
                ref realm,
                ref details,
            } => json!([code, realm, details]),

            Authenticate {
                ref signature,
                ref extra,
            } => json!([code, signature, extra]),

            Goodbye {
                ref details,
                ref reason,
            } => json!([code, details, reason]),

            Error {
                ref request_type,
                ref request,
                ref details,
                ref error,
                ref arguments,
                ref arguments_kw,
            } => {
                if let Some(ref arguments) = arguments {
                    if let Some(ref arguments_kw) = arguments_kw {
                        json!([
                            code,
                            request_type,
                            request,
                            details,
                            error,
                            arguments,
                            arguments_kw,
                        ])
                    } else {
                        json!([code, request_type, request, details, error, arguments])
                    }
                } else {
                    json!([code, request_type, request, details, error])
                }
            }

            #[cfg(feature = "publisher")]
            Publish {
                ref request,
                ref options,
                ref topic,
                ref arguments,
                ref arguments_kw,
            } => {
                if let Some(ref arguments) = arguments {
                    if let Some(ref arguments_kw) = arguments_kw {
                        json!([code, request, options, topic, arguments, arguments_kw])
                    } else {
                        json!([code, request, options, topic, arguments])
                    }
                } else {
                    json!([code, request, options, topic])
                }
            }

            #[cfg(feature = "subscriber")]
            Subscribe {
                ref request,
                ref options,
                ref topic,
            } => json!([code, request, options, topic]),

            #[cfg(feature = "subscriber")]
            Unsubscribe {
                ref request,
                ref subscription,
            } => json!([code, request, subscription]),

            #[cfg(feature = "caller")]
            Call {
                ref request,
                ref options,
                ref procedure,
                ref arguments,
                ref arguments_kw,
            } => {
                if let Some(ref arguments) = arguments {
                    if let Some(ref arguments_kw) = arguments_kw {
                        json!([code, request, options, procedure, arguments, arguments_kw])
                    } else {
                        json!([code, request, options, procedure, arguments])
                    }
                } else {
                    json!([code, request, options, procedure])
                }
            }

            #[cfg(feature = "callee")]
            Register {
                ref request,
                ref options,
                ref procedure,
            } => json!([code, request, options, procedure]),

            #[cfg(feature = "callee")]
            Unregister {
                ref request,
                ref registration,
            } => json!([code, request, registration]),

            #[cfg(feature = "callee")]
            Yield {
                ref request,
                ref options,
                ref arguments,
                ref arguments_kw,
            } => {
                if let Some(ref arguments) = arguments {
                    if let Some(ref arguments_kw) = arguments_kw {
                        json!([code, request, options, arguments, arguments_kw])
                    } else {
                        json!([code, request, options, arguments])
                    }
                } else {
                    json!([code, request, options])
                }
            }
        }
    }
}
