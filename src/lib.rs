use std::collections::HashMap;

use futures::{SinkExt, StreamExt};

mod protocol;

pub async fn start() {
    // Configure

    //let url = url::Url::parse("ws://127.0.0.1:8080/ws").unwrap();
    let url = url::Url::parse("wss://dots.org.ua/wamp").unwrap();

    // Connect
    let mut req = tungstenite::handshake::client::Request::from(url);
    req.add_protocol("wamp.2.json".into());
    req.add_protocol("wamp.2.msgpack".into());
    let (ws_stream, _) = tokio_tungstenite::connect_async(req)
        .await
        .expect("Failed to connect");
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Setup workers
    let (mut wamp_protocol_sender_tx, mut wamp_protocol_sender_rx) =
        futures::channel::mpsc::channel::<tungstenite::Message>(1);

    let (wamp_messages_sender_tx, mut wamp_messages_sender_rx) =
        futures::channel::mpsc::channel::<protocol::ClientMessage>(1);

    let (wamp_serializer_tx, mut wamp_serializer_rx) =
        futures::channel::mpsc::channel::<Vec<u8>>(1);

    let (wamp_events_tx, mut wamp_events_rx) =
        futures::channel::mpsc::channel::<serde_json::Value>(1);

    type WampResolver<T> = futures::channel::oneshot::Sender<T>;
    type WampInvocationFunction = Box<
        fn(HashMap<String, String>) -> futures::future::BoxFuture<'static, HashMap<String, String>>,
    >;

    struct WampRegisterInfo {
        resolver: WampResolver<()>,
        handler: WampInvocationFunction,
    };

    struct WampCallInfo {
        resolver: WampResolver<()>,
    }

    type TypedPendingRequests<T> =
        tokio::sync::Mutex<HashMap<protocol::Id<protocol::id::SessionScope>, T>>;

    #[derive(Default)]
    struct PendingRequests {
        registrations: TypedPendingRequests<WampRegisterInfo>,
        calls: TypedPendingRequests<WampCallInfo>,
    }

    type WampPendingRequests = std::sync::Arc<PendingRequests>;

    type WampRegisteredFunctions = std::sync::Arc<
        tokio::sync::Mutex<
            HashMap<protocol::Id<protocol::id::RouterScope>, WampInvocationFunction>,
        >,
    >;

    let pending_requests: WampPendingRequests = std::sync::Arc::new(PendingRequests::default());

    let registered_functions: WampRegisteredFunctions =
        std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    {
        let mut wamp_protocol_sender_tx = wamp_protocol_sender_tx.clone();
        let mut wamp_serializer_tx = wamp_serializer_tx.clone();
        tokio::spawn(async move {
            while let Some(ws_msg) = ws_rx.next().await {
                let ws_msg = ws_msg.expect("Failed to get response");
                //eprintln!("NEW DATA: {:?}", ws_msg);
                let data = match ws_msg {
                    tungstenite::Message::Text(data) => data.into_bytes(),
                    tungstenite::Message::Ping(data) => {
                        wamp_protocol_sender_tx
                            .send(tungstenite::Message::Pong(data))
                            .await
                            .expect("PONG");
                        continue;
                    }
                    ws_msg => {
                        unimplemented!("unknown WS message type: {:?}", ws_msg);
                    }
                };
                wamp_serializer_tx.send(data).await.expect("event pushed");
            }
        });
    }

    {
        let mut wamp_events_tx = wamp_events_tx.clone();
        tokio::spawn(async move {
            while let Some(data) = wamp_serializer_rx.next().await {
                let msg = match serde_json::from_slice::<serde_json::Value>(&data) {
                    Ok(msg) => msg,
                    Err(error) => {
                        println!(
                            "JSON parsing error: {:?}: {:?}",
                            error,
                            String::from_utf8(data)
                        );
                        continue;
                    }
                };

                eprintln!("NEW MESSAGE: {:?}", msg);
                wamp_events_tx.send(msg).await.expect("event pushed");
            }
        });
    }

    {
        let wamp_messages_sender_tx = wamp_messages_sender_tx.clone();
        let pending_requests = std::sync::Arc::clone(&pending_requests);

        async fn handle_challenge(
            mut wamp_messages_sender_tx: futures::channel::mpsc::Sender<protocol::ClientMessage>,
            _args: &[serde_json::Value],
        ) {
            wamp_messages_sender_tx
                .send(protocol::ClientMessage::Authenticate {
                    signature: "test".to_owned(),
                    extra: std::collections::HashMap::new(),
                })
                .await
                .expect("reply");
        }

        async fn handle_registered(
            mut _wamp_messages_sender_tx: futures::channel::mpsc::Sender<protocol::ClientMessage>,
            pending_requests: WampPendingRequests,
            registered_functions: WampRegisteredFunctions,
            args: &[serde_json::Value],
        ) {
            let request_id = if let Some(request_id) = args[0].as_u64() {
                protocol::Id::<protocol::id::SessionScope>::from_raw_value(request_id)
            } else {
                println!("unexpected value as request id: {:?}", args[0]);
                return;
            };
            let registration_id = if let Some(registration_id) = args[1].as_u64() {
                protocol::Id::<protocol::id::RouterScope>::from_raw_value(registration_id)
            } else {
                println!("unexpected value as Register id: {:?}", args[1]);
                return;
            };

            let register_info = if let Some(register_info) = pending_requests
                .registrations
                .lock()
                .await
                .remove(&request_id)
            {
                register_info
            } else {
                println!(
                    "unexpected request id for a registered function: {:?}",
                    request_id
                );
                return;
            };
            {
                registered_functions
                    .lock()
                    .await
                    .insert(registration_id, register_info.handler);
            }
            register_info.resolver.send(()).expect("resolved reg");
        }

        async fn handle_invocation(
            mut wamp_messages_sender_tx: futures::channel::mpsc::Sender<protocol::ClientMessage>,
            registered_functions: WampRegisteredFunctions,
            args: &[serde_json::Value],
        ) {
            eprintln!("HADNLING INVO...");
            let request_id = if let Some(request_id) = args[0].as_u64() {
                protocol::Id::<protocol::id::RouterScope>::from_raw_value(request_id)
            } else {
                println!("unexpected value as request id: {:?}", args[0]);
                return;
            };
            let registration_id = if let Some(registration_id) = args[1].as_u64() {
                protocol::Id::<protocol::id::RouterScope>::from_raw_value(registration_id)
            } else {
                println!("unexpected value as Register id: {:?}", args[1]);
                return;
            };

            eprintln!("GETTING HANDLER");
            let handler =
                if let Some(handler) = registered_functions.lock().await.get(&registration_id) {
                    handler.clone()
                } else {
                    println!(
                        "there is no registered function with id: {:?}",
                        registration_id
                    );
                    return;
                };
            let f: futures::future::BoxFuture<_> = (handler)(HashMap::new());
            tokio::spawn(async move {
                eprintln!("INVOKNIG");
                f.await;
                let wamp_result = {
                    protocol::ClientMessage::Yield {
                        request: request_id,
                        options: HashMap::new(),
                        arguments: Some(vec![protocol::TransportableValue::String(
                            "resp".to_owned(),
                        )]),
                        arguments_kw: None,
                    }
                };
                eprintln!("SENDING RESULT...");
                wamp_messages_sender_tx
                    .send(wamp_result)
                    .await
                    .expect("result sent");
            });
        }

        async fn handle_result(
            mut _wamp_messages_sender_tx: futures::channel::mpsc::Sender<protocol::ClientMessage>,
            pending_requests: WampPendingRequests,
            args: &[serde_json::Value],
        ) {
            let request_id = if let Some(request_id) = args[0].as_u64() {
                protocol::Id::<protocol::id::SessionScope>::from_raw_value(request_id)
            } else {
                println!("unexpected value as request id: {:?}", args[0]);
                return;
            };

            let call_info = {
                let mut calls = pending_requests.calls.lock().await;
                println!("-C: {}", calls.len());
                if let Some(call_info) = calls.remove(&request_id) {
                    call_info
                } else {
                    println!(
                        "unexpected request id for a registered function: {:?}",
                        request_id
                    );
                    return;
                }
            };
            call_info.resolver.send(()).expect("resolved call");
        }

        tokio::spawn(async move {
            let pending_requests = pending_requests;
            let registered_functions = registered_functions;
            while let Some(wamp_event) = wamp_events_rx.next().await {
                let msg = if let serde_json::Value::Array(msg) = wamp_event {
                    msg
                } else {
                    println!("received not an array");
                    continue;
                };
                let code = if let Some(code) = msg[0].as_u64() {
                    code
                } else {
                    println!("code is not u64");
                    continue;
                };
                use crate::protocol::router_message::RouterMessage;

                //eprintln!("HANDLING...");
                match code {
                    protocol::router_message::Challenge::MSG_CODE => {
                        handle_challenge(wamp_messages_sender_tx.clone(), &msg[1..]).await;
                    }
                    protocol::router_message::Registered::MSG_CODE => {
                        handle_registered(
                            wamp_messages_sender_tx.clone(),
                            std::sync::Arc::clone(&pending_requests),
                            std::sync::Arc::clone(&registered_functions),
                            &msg[1..],
                        )
                        .await;
                    }
                    protocol::router_message::Invocation::MSG_CODE => {
                        handle_invocation(
                            wamp_messages_sender_tx.clone(),
                            std::sync::Arc::clone(&registered_functions),
                            &msg[1..],
                        )
                        .await;
                    }
                    protocol::router_message::Result::MSG_CODE => {
                        handle_result(
                            wamp_messages_sender_tx.clone(),
                            std::sync::Arc::clone(&pending_requests),
                            &msg[1..],
                        )
                        .await;
                    }
                    _ => {
                        println!("unknown message code: {}", code);
                    }
                }
                //eprintln!("HANDLED");
            }
        });
    }

    tokio::spawn(async move {
        while let Some(msg) = wamp_messages_sender_rx.next().await {
            let msg = tungstenite::Message::text(msg.to_json().to_string());
            eprintln!("SENDING WAMP MSG...");
            wamp_protocol_sender_tx.send(msg).await.expect("sent");
            eprintln!("SENT WAMP MSG");
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = wamp_protocol_sender_rx.next().await {
            eprintln!("SENDING DATA...");
            ws_tx.send(msg).await.expect("sent");
            eprintln!("SENT DATA");
        }
    });

    async fn init(
        mut wamp_messages_sender_tx: futures::channel::mpsc::Sender<protocol::ClientMessage>,
    ) {
        let wamp_hello = {
            use protocol::transportable_value::{Dict, TransportableValue};
            let mut details = Dict::new();
            let mut roles = Dict::new();
            roles.insert("caller".to_owned(), TransportableValue::Dict(Dict::new()));
            roles.insert("callee".to_owned(), TransportableValue::Dict(Dict::new()));
            roles.insert(
                "publisher".to_owned(),
                TransportableValue::Dict(Dict::new()),
            );
            roles.insert(
                "subscriber".to_owned(),
                TransportableValue::Dict(Dict::new()),
            );
            details.insert("roles".to_owned(), TransportableValue::Dict(roles));
            //*
            details.insert(
                "authmethods".to_owned(),
                TransportableValue::List(vec![TransportableValue::String("ticket".to_owned())]),
            );
            details.insert(
                "authid".to_owned(),
                TransportableValue::String("dots-test".to_owned()),
            );
            // */
            protocol::ClientMessage::Hello {
                realm: protocol::Uri::relaxed("dots").unwrap(),
                details,
            }
        };
        /*
        let msg = tungstenite::Message::text(
            //r#"[1,"near-explorer",{"roles":{"caller":{},"callee":{},"publisher":{},"subscriber":{}}}]"#,
            //r#"[1,"near-explorer",{"roles":{"caller":{},"callee":{},"publisher":{},"subscriber":{}},"authmethods":["ticket"],"authid":"near-explorer-backend"}]"#,
            //r#"[1,"dots",{"roles":{"caller":{},"callee":{},"publisher":{},"subscriber":{}},"authmethods":["ticket"],"authid":"dots-backend"}]"#,
            wamp_hello.to_json().to_string(),
        );*/
        //ws_tx.send(msg).await.expect("Failed to send request");
        wamp_messages_sender_tx
            .send(wamp_hello)
            .await
            .expect("hello");

        tokio::timer::delay_for(std::time::Duration::from_secs(1)).await;
    }

    async fn register(
        mut wamp_messages_sender_tx: futures::channel::mpsc::Sender<protocol::ClientMessage>,
        pending_requests: WampPendingRequests,
        procedure: protocol::Uri,
        handler: WampInvocationFunction,
    ) {
        let request = protocol::Id::<protocol::id::SessionScope>::next();
        let wamp_register = {
            use protocol::transportable_value::Dict;

            protocol::ClientMessage::Register {
                request,
                options: Dict::new(),
                procedure,
            }
        };
        let (resolver_tx, resolver_rx) = futures::channel::oneshot::channel();
        {
            pending_requests.registrations.lock().await.insert(
                request,
                WampRegisterInfo {
                    resolver: resolver_tx,
                    handler,
                },
            );
        }
        wamp_messages_sender_tx
            .send(wamp_register)
            .await
            .expect("register message sent");
        resolver_rx.await.expect("registration complete callback");
    }

    async fn call(
        mut wamp_messages_sender_tx: futures::channel::mpsc::Sender<protocol::ClientMessage>,
        pending_requests: WampPendingRequests,
        procedure: protocol::Uri,
    ) {
        let request = protocol::Id::<protocol::id::SessionScope>::next();
        let wamp_call = {
            use protocol::transportable_value::{Dict, TransportableValue};

            protocol::ClientMessage::Call {
                request,
                options: Dict::new(),
                procedure,
                arguments: Some(vec![TransportableValue::String("frol".to_owned())]),
                arguments_kw: None,
            }
        };
        let (resolver_tx, resolver_rx) = futures::channel::oneshot::channel();
        {
            let mut calls = pending_requests.calls.lock().await;
            println!("C: {}", calls.len());
            calls.insert(
                request,
                WampCallInfo {
                    resolver: resolver_tx,
                },
            );
        }
        wamp_messages_sender_tx
            .send(wamp_call)
            .await
            .expect("call message is sent");
        resolver_rx.await.expect("call callback is called");
    }

    async fn handler_test_rust(_args: HashMap<String, String>) -> HashMap<String, String> {
        //println!("handling...");
        //tokio::timer::delay_for(std::time::Duration::from_secs(1)).await;
        //println!("handled");
        let x = HashMap::new();
        //x.insert("a".to_owned(), "b".to_owned());
        x
    }

    init(wamp_messages_sender_tx.clone()).await;
    register(
        wamp_messages_sender_tx.clone(),
        pending_requests.clone(),
        protocol::Uri::relaxed("com.demo.test-rust").unwrap(),
        Box::new(move |args| Box::pin(handler_test_rust(args))),
    )
    .await;

    eprintln!("qq");
    call(
        wamp_messages_sender_tx.clone(),
        pending_requests.clone(),
        protocol::Uri::relaxed("com.demo.test-rust").unwrap(),
    )
    .await;

    tokio::timer::delay_for(std::time::Duration::from_secs(1)).await;
    eprintln!("start");
    for _ in 0..1 {
        let wamp_messages_sender_tx = wamp_messages_sender_tx.clone();
        let pending_requests = pending_requests.clone();
        tokio::spawn(async move {
            let mut queue = futures::stream::futures_unordered::FuturesUnordered::new();
            let start = std::time::Instant::now();
            for _ in 0..1000 {
                queue.push(call(
                    wamp_messages_sender_tx.clone(),
                    pending_requests.clone(),
                    protocol::Uri::relaxed("com.demo.test-rust").unwrap(),
                ));
            }
            while let Some(_) = queue.next().await {}
            eprintln!("end: {:?}", start.elapsed());
        });
    }

    tokio::timer::delay_for(std::time::Duration::from_secs(600)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        assert!(false);
    }
}
