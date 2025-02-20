use std::{
    collections::{HashMap, HashSet},
    io,
};

use serde_json::json;
use tokio::sync::{mpsc, oneshot};

use crate::commands::{DeviceCommand, ServerCommand, UserCommand};
use crate::entities::{Device, DeviceId, DeviceState, User, UserId};

#[derive(Debug)]
pub struct ConnectionServer {
    devices: HashMap<DeviceId, Device>,
    users: HashMap<UserId, User>,
    user_connections: HashMap<UserId, HashSet<DeviceId>>,
    device_connection: HashMap<DeviceId, UserId>,
    cmd_rx: mpsc::UnboundedReceiver<ServerCommand>,
}

impl ConnectionServer {
    pub fn new() -> (Self, ConnectionServerHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            ConnectionServer {
                devices: HashMap::new(),
                users: HashMap::new(),
                user_connections: HashMap::new(),
                device_connection: HashMap::new(),
                cmd_rx,
            },
            ConnectionServerHandle { cmd_tx },
        )
    }

    async fn notify_user(&self, user_id: &UserId, cmd: UserCommand<'_>) {
        if let Some(user) = self.users.get(user_id) {
            let _ = user.conn_tx.send(json!(cmd).to_string());
        }
    }

    async fn notify_users(&self, cmd: UserCommand<'_>) {
        let cmd = json!(cmd).to_string();

        self.users.values().for_each(|user| {
            let _ = user.conn_tx.send(cmd.clone());
        });
    }

    async fn notify_device(&self, device_id: &DeviceId, cmd: DeviceCommand) {
        if let Some(device) = self.devices.get(device_id) {
            let _ = device.conn_tx.send(json!(cmd).to_string());
        }
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                ServerCommand::RegisterUser { conn_tx, res_tx } => {
                    let new_user = User::new(conn_tx);
                    let new_user_id = new_user.id;
                    self.users.insert(new_user_id, new_user);
                    let _ = res_tx.send(new_user_id);
                    self.notify_user(
                        &new_user_id,
                        UserCommand::UpdateDevices {
                            devices: self
                                .devices
                                .values()
                                .filter(|device| device.state != DeviceState::Connected)
                                .collect::<Vec<&Device>>(),
                        },
                    )
                    .await;
                }

                ServerCommand::UnregisterUser { user_id } => {
                    self.users.remove(&user_id);
                    if let Some(device_ids) = self.user_connections.remove(&user_id) {
                        device_ids.iter().for_each(|device_id| {
                            self.device_connection.remove(device_id);
                            if let Some(device) = self.devices.get_mut(device_id) {
                                device.state = DeviceState::Open;
                            }
                        });
                        self.notify_users(UserCommand::UpdateDevices {
                            devices: self
                                .devices
                                .values()
                                .filter(|device| device_ids.contains(&device.id))
                                .collect::<Vec<&Device>>(),
                        })
                        .await;
                    };
                }

                ServerCommand::RegisterDevice {
                    name,
                    conn_tx,
                    res_tx,
                } => {
                    let new_device = Device::new(name, conn_tx);
                    let new_device_id = new_device.id;
                    self.notify_users(UserCommand::UpdateDevices {
                        devices: vec![&new_device],
                    })
                    .await;
                    self.devices.insert(new_device_id, new_device);
                    let _ = res_tx.send(new_device_id);
                }

                ServerCommand::UnregisterDevice { device_id } => {
                    self.devices.remove(&device_id);

                    if let Some(user_id) = self.device_connection.remove(&device_id) {
                        if let Some(connections) = self.user_connections.get_mut(&user_id) {
                            connections.remove(&device_id);
                            self.notify_user(
                                &user_id,
                                UserCommand::RemoveConnectedDevice {
                                    device_id: &device_id,
                                },
                            )
                            .await;
                        }
                    } else {
                        self.notify_users(UserCommand::RemoveDevice {
                            device_id: &device_id,
                        })
                        .await;
                    };
                }

                ServerCommand::Connect { user_id, device_id } => {
                    if let Some(device) = self.devices.get_mut(&device_id) {
                        if device.state == DeviceState::Open {
                            device.state = DeviceState::Connected;

                            if let Some(connections) = self.user_connections.get_mut(&user_id) {
                                connections.insert(device_id);
                            } else {
                                self.user_connections
                                    .insert(user_id, HashSet::from([device_id]));
                            }

                            self.device_connection.insert(device_id, user_id);
                            self.notify_users(UserCommand::RemoveDevice {
                                device_id: &device_id,
                            })
                            .await;

                            if let Some(device) = self.devices.get(&device_id) {
                                self.notify_user(
                                    &user_id,
                                    UserCommand::UpdateConnectedDevice { device },
                                )
                                .await;
                            }
                        }
                    }
                }

                ServerCommand::Disconnect { user_id, device_id } => {}

                ServerCommand::UserSignaling {
                    user_id,
                    device_id,
                    signal,
                } => {
                    if let Some(connections) = self.user_connections.get(&user_id) {
                        if connections.get(&device_id).is_some() {
                            self.notify_device(&device_id, DeviceCommand::UserSignaling { signal })
                                .await;
                        }
                    }
                }

                ServerCommand::DeviceSignaling { device_id, signal } => {
                    if let Some(user_id) = self.device_connection.get(&device_id) {
                        self.notify_user(
                            user_id,
                            UserCommand::DeviceSignaling {
                                device_id: &device_id,
                                signal,
                            },
                        )
                        .await;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionServerHandle {
    cmd_tx: mpsc::UnboundedSender<ServerCommand>,
}

impl ConnectionServerHandle {
    pub async fn register_user(&self, conn_tx: mpsc::UnboundedSender<String>) -> UserId {
        let (res_tx, res_rx) = oneshot::channel();

        let _ = self
            .cmd_tx
            .send(ServerCommand::RegisterUser { res_tx, conn_tx });

        // unwrap: i dont care
        res_rx.await.unwrap()
    }

    pub fn unregister_user(&self, user_id: UserId) {
        let _ = self.cmd_tx.send(ServerCommand::UnregisterUser { user_id });
    }

    pub async fn register_device(
        &self,
        name: String,
        conn_tx: mpsc::UnboundedSender<String>,
    ) -> DeviceId {
        let (res_tx, res_rx) = oneshot::channel();

        let _ = self.cmd_tx.send(ServerCommand::RegisterDevice {
            name,
            conn_tx,
            res_tx,
        });

        // unwrap: i dont care
        res_rx.await.unwrap()
    }

    pub fn unregister_device(&self, device_id: DeviceId) {
        self.cmd_tx
            .send(ServerCommand::UnregisterDevice { device_id })
            .unwrap();
    }

    pub async fn connect(&self, user_id: UserId, device_id: DeviceId) {
        let _ = self
            .cmd_tx
            .send(ServerCommand::Connect { user_id, device_id });
    }

    pub async fn user_signaling(
        &self,
        user_id: UserId,
        device_id: DeviceId,
        signal: serde_json::Value,
    ) {
        let _ = self.cmd_tx.send(ServerCommand::UserSignaling {
            user_id,
            device_id,
            signal,
        });
    }

    pub async fn device_signaling(&self, device_id: DeviceId, signal: serde_json::Value) {
        let _ = self
            .cmd_tx
            .send(ServerCommand::DeviceSignaling { device_id, signal });
    }
}
