use std::{
    pin::{pin, Pin},
    time::{Duration, Instant},
};

use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    Future, StreamExt as _,
};
use tokio::{sync::mpsc, task::spawn_local, time::interval};

use crate::commands::ServerCommandJson;
use crate::entities::{DeviceId, UserId};
use crate::server::ConnectionServerHandle;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

macro_rules! define_ws_handler {
    ($fn_name:ident, $register_fn:ident, $unregister_fn:ident, $process_cmd_fn:expr $(, $extra_param:ident : $extra_ty:ty)?) => {
        async fn $fn_name(
            $( $extra_param: $extra_ty, )?
            mut session: actix_ws::Session,
            msg_stream: actix_ws::MessageStream,
            connection_server: web::Data<ConnectionServerHandle>,
        ) {
            let mut last_heartbeat = Instant::now();
            let mut interval = interval(HEARTBEAT_INTERVAL);
            let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

            let entity_id = connection_server.$register_fn($( $extra_param, )? conn_tx).await;

            let msg_stream = msg_stream
                .max_frame_size(128 * 1024)
                .aggregate_continuations()
                .max_continuation_size(2 * 1024 * 1024);

            let mut msg_stream = pin!(msg_stream);

            let close_reason = loop {
                let tick = pin!(interval.tick());
                let msg_rx = pin!(conn_rx.recv());
                let messages = pin!(select(msg_stream.next(), msg_rx));

                match select(messages, tick).await {
                    Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => match msg {

                        AggregatedMessage::Ping(bytes) => {
                            last_heartbeat = Instant::now();
                            session.pong(&bytes).await.unwrap();
                        }

                        AggregatedMessage::Pong(_) => {
                            last_heartbeat = Instant::now();
                        }

                        // cmd send from user
                        AggregatedMessage::Text(cmd) => {
                            $process_cmd_fn(connection_server.clone(), entity_id, cmd.to_string())
                                .await;
                        }

                        AggregatedMessage::Binary(_) => {}

                        AggregatedMessage::Close(reason) => break reason,
                    },

                    Either::Left((Either::Left((Some(Err(_)), _)), _)) => break None,

                    Either::Left((Either::Left((None, _)), _)) => break None,

                    // cmd send to user
                    Either::Left((Either::Right((Some(cmd), _)), _)) => {
                        session.text(cmd).await.unwrap();
                    }

                    Either::Left((Either::Right((None, _)), _)) => unreachable!(
                        "all connection message senders were dropped; chat server may have panicked"
                    ),

                    Either::Right((_inst, _)) => {
                        if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                            break None;
                        }
                        let _ = session.ping(b"").await;
                    }
                };
            };

            connection_server.$unregister_fn(entity_id);
            let _ = session.close(close_reason).await;
        }
    };
}

define_ws_handler!(
    user_ws,
    register_user,
    unregister_user,
    &Box::new(
        |connection_server: web::Data<ConnectionServerHandle>,
         user_id: UserId,
         text: String|
         -> Pin<Box<dyn Future<Output = ()> + Send>> {
            Box::pin(async move {
                let command: ServerCommandJson = serde_json::from_str(&text).unwrap();

                match command {
                    ServerCommandJson::Connect { device_id } => {
                        connection_server.connect(user_id, device_id).await;
                    }

                    ServerCommandJson::UserSignaling { device_id, signal } => {
                        connection_server
                            .user_signaling(user_id, device_id, signal)
                            .await;
                    }

                    ServerCommandJson::Disconnect { device_id } => {
                        todo!();
                    }

                    _ => todo!("handle unknown command (error)"),
                }
            })
        }
    )
);

define_ws_handler!(
    device_ws,
    register_device,
    unregister_device,
    &Box::new(
        |connection_server: web::Data<ConnectionServerHandle>,
         device_id: DeviceId,
         text: String|
         -> Pin<Box<dyn Future<Output = ()> + Send>> {
            Box::pin(async move {
                let command: ServerCommandJson = serde_json::from_str(&text).unwrap();

                match command {
                    ServerCommandJson::DeviceSignaling { signal } => {
                        connection_server.device_signaling(device_id, signal).await;
                    }

                    _ => todo!(),
                }
            })
        }
    ),
        name: String
);

#[get("/signaling/register/user")]
pub async fn register_user(
    req: HttpRequest,
    stream: web::Payload,
    connection_server: web::Data<ConnectionServerHandle>,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    spawn_local(user_ws(session, msg_stream, connection_server));

    Ok(res)
}

#[get("/signaling/register/device/{name}")]
pub async fn register_device(
    name: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    connection_server: web::Data<ConnectionServerHandle>,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    spawn_local(device_ws(
        name.into_inner(),
        session,
        msg_stream,
        connection_server,
    ));

    Ok(res)
}
