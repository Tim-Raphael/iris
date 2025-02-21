use std::{
    collections::{HashMap, HashSet},
    io,
};

use serde_json::json;
use tokio::sync::{mpsc, oneshot};

use crate::commands::{DeviceCommand, ServerCommand, UserCommand};
use crate::entities::{Device, DeviceId, DeviceState, User, UserId};

#[derive(Debug, Default)]
struct ConnectionManager {
    user_connections: HashMap<UserId, HashSet<DeviceId>>,
    device_connection: HashMap<DeviceId, UserId>,
    open_devices: HashSet<DeviceId>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&mut self, user_id: &UserId, device_id: &DeviceId) -> Result<(), String> {
        self.open_devices.remove(device_id);
        if let Some(existing_user) = self.device_connection.get(device_id) {
            if existing_user != user_id {
                return Err(format!(
                    "Device {} is already assigned to User {}",
                    device_id, existing_user
                ));
            }
        }
        self.user_connections
            .entry(*user_id)
            .or_insert_with(HashSet::new)
            .insert(*device_id);
        self.device_connection.insert(*device_id, *user_id);
        Ok(())
    }

    pub fn disconnect(&mut self, user_id: &UserId, device_id: &DeviceId) -> Result<(), String> {
        if let Some(user_id) = self.device_connection.remove(device_id) {
            if let Some(device_set) = self.user_connections.get_mut(&user_id) {
                device_set.remove(&device_id);
                if device_set.is_empty() {
                    self.user_connections.remove(&user_id);
                }
            }
            self.open_devices.insert(*device_id);
            Ok(())
        } else {
            self.open_devices.insert(*device_id);
            Err(format!("Device {} was not assigned to any user", device_id))
        }
    }

    pub fn remove_device(&mut self, device_id: &DeviceId) -> Option<UserId> {
        if let Some(user_id) = self.device_connection.remove(device_id) {
            self.user_connections.remove(&device_id);
            return Some(user_id);
        }
        None
    }

    pub fn remove_user(&mut self, user_id: &UserId) -> Option<Vec<DeviceId>> {
        if let Some(device_set) = self.user_connections.remove(user_id) {
            let mut opened_devices = Vec::with_capacity(device_set.len());
            device_set.iter().for_each(|device_id| {
                self.device_connection.remove(device_id);
                self.open_devices.insert(*device_id);
                opened_devices.push(*device_id);
            });
            return Some(opened_devices);
        }
        None
    }

    pub fn is_user_device(&self, user_id: DeviceId, device_id: DeviceId) -> bool {
        todo!()
    }

    pub fn get_user_devices(&self, user_id: UserId) -> Option<Vec<&DeviceId>> {
        let user_connections = self.user_connections.get(&user_id)?;
        Some(user_connections.iter().collect::<Vec<&DeviceId>>())
    }

    pub fn get_device_user(&self, device_id: DeviceId) -> Option<&UserId> {
        self.device_connection.get(&device_id)
    }

    pub fn get_open_devices(&self) -> Vec<&DeviceId> {
        self.open_devices.iter().collect::<Vec<&DeviceId>>()
    }
}

#[derive(Debug)]
pub struct ConnectionServer {
    devices: HashMap<DeviceId, Device>,
    users: HashMap<UserId, User>,
    connections: ConnectionManager,
    cmd_rx: mpsc::UnboundedReceiver<ServerCommand>,
}

impl ConnectionServer {
    pub fn new() -> (Self, ConnectionServerHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            ConnectionServer {
                devices: HashMap::new(),
                users: HashMap::new(),
                connections: ConnectionManager::new(),
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
                    self.notify_user(
                        &new_user_id,
                        UserCommand::UpdateDevices {
                            devices: self
                                .connections
                                .get_open_devices()
                                .into_iter()
                                .filter_map(|device_id| self.devices.get(device_id))
                                .collect::<Vec<&Device>>(),
                        },
                    );
                    let _ = res_tx.send(new_user_id);
                }

                ServerCommand::UnregisterUser { user_id } => {
                    self.users.remove(&user_id);
                    if let Some(device_ids) = self.connections.remove_user(&user_id) {
                        self.notify_users(UserCommand::UpdateDevices {
                            devices: device_ids
                                .iter()
                                .filter_map(|device_id| self.devices.get(device_id))
                                .collect::<Vec<&Device>>(),
                        });
                    }
                }

                ServerCommand::RegisterDevice {
                    name,
                    conn_tx,
                    res_tx,
                } => {
                    let new_device = Device::new(name, conn_tx);
                    let new_device_id = new_device.id;
                    self.devices.insert(new_device_id, new_device);
                    self.connections.open_devices.insert(new_device_id);
                    self.notify_users(UserCommand::UpdateDevices {
                        devices: vec![&new_device],
                    });
                    let _ = res_tx.send(new_device_id);
                }

                ServerCommand::UnregisterDevice { device_id } => {
                    self.devices.remove(&device_id);
                    if let Some(user_id) = self.connections.remove_device(&device_id) {
                        self.notify_user(
                            &user_id,
                            UserCommand::RemoveConnectedDevice {
                                device_id: &device_id,
                            },
                        );
                    } else {
                        self.notify_users(UserCommand::RemoveDevice {
                            device_id: &device_id,
                        });
                    }
                }

                ServerCommand::Connect { user_id, device_id } => {
                    if let Some(device) = self.devices.get_mut(&device_id) {
                        device.state = DeviceState::Connected;
                        if let Err(err) = self.connections.connect(&user_id, &device_id) {
                            self.notify_user(&user_id, UserCommand::Error { message: err });
                            todo!("implement a sync command");
                        }
                    }
                }

                ServerCommand::Disconnect { user_id, device_id } => {
                    self.notify_user(
                        &user_id,
                        UserCommand::RemoveConnectedDevice {
                            device_id: &device_id,
                        },
                    );
                    if let Err(err) = self.connections.disconnect(&user_id, &device_id) {
                        self.notify_user(&user_id, UserCommand::Error { message: err });
                        todo!("implement a sync command");
                    } else if let Some(device) = self.devices.get_mut(&device_id) {
                        device.state = DeviceState::Open;
                        if let Some(device) = self.devices.get(&device_id) {
                            self.notify_users(UserCommand::UpdateDevices {
                                devices: vec![device],
                            });
                        }
                    }
                }

                //////
                ServerCommand::UserSignaling {
                    user_id,
                    device_id,
                    signal,
                } => {
                    if let Some(connections) = self.connections.get(&user_id) {
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
