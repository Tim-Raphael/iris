![Image of Retro Car with Iris-Logo in front](./design/banner_branding.jpg)

<div align="center">

# `🪻 Iris`

**A user-friendly interface that lets you remotely control small cars.**

</div>

## 🚧 Key Features
This project is still under development, but it already supports the following
feartures:

- starting the WebRTC signaling process and rendering the incoming stream
- registration as a User or Device
- connecting to a Device

## 🚀 Getting Started
You can start up this project by running `nix-shell` alternatively you could: 

```bash
cd web/
pnpm run dev
cd ../iris/
cargo run
``` 

## 💻 Structure 
The entire application based around web sockets. You can register
either as a [Device](#device) or [User](#user). When registered the 
ConnectionServer keeps track of your current state and all connections between 
the User(1) and Devices(n).

### Commands
This application is structured around commands. Each participant can send and
receive commands via their ws connection. The ConnectionServer handles these 
commands.

<details>
<summary>ServerCommand</summary>

These are the commands the ConnectionServer can handle, but most of them are not
directly send by the user or device. The registration of a user, for example, is 
done by the `user_ws_hander`.

| Variant               | Parameter           | Type                                   | Description                                                      |
|-----------------------|---------------------|----------------------------------------|------------------------------------------------------------------|
| `RegisterUser`        | `conn_tx`            | `mpsc::UnboundedSender<String>`        | Transmission channel for user communication.                    |
|                       | `res_tx`             | `oneshot::Sender<UserId>`              | One-shot channel to send back the registered `UserId`.           |
| `UnregisterUser`      | `user_id`            | `UserId`                               | ID of the user to unregister.                                    |
| `RegisterDevice`      | `name`               | `String`                               | Name of the device being registered.                             |
|                       | `conn_tx`            | `mpsc::UnboundedSender<String>`        | Transmission channel for device communication.                   |
|                       | `res_tx`             | `oneshot::Sender<DeviceId>`            | One-shot channel to send back the registered `DeviceId`.         |
| `UnregisterDevice`    | `device_id`          | `DeviceId`                             | ID of the device to unregister.                                  |
| `Connect`             | `user_id`            | `UserId`                               | ID of the user initiating connection.                            |
|                       | `device_id`          | `DeviceId`                             | ID of the device being connected.                                |
| `Disconnect`          | `user_id`            | `UserId`                               | ID of the user disconnecting.                                    |
|                       | `device_id`          | `DeviceId`                             | ID of the device being disconnected.                             |
| `UserSignaling`       | `user_id`            | `UserId`                               | ID of the user sending a signaling message.                      |
|                       | `device_id`          | `DeviceId`                             | ID of the target device.                                         |
|                       | `signal`             | `serde_json::Value`                    | Signal data (e.g., RTCSessionDescription or RTCIceCandidate).    |
| `DeviceSignaling`     | `device_id`          | `DeviceId`                             | ID of the device sending a signaling message.                    |
|                       | `signal`             | `serde_json::Value`                    | Signal data (e.g., RTCSessionDescription or RTCIceCandidate).    |
</details>

<details>
<summary>ServerCommandJson</summary>

These are all commands that either the user or the device send via their ws which
gets parsed by their respective ws_handler.

| Variant               | Parameter           | Type                                   | Description                                                      |
|-----------------------|---------------------|----------------------------------------|------------------------------------------------------------------|
| `Connect`             | `device_id`          | `DeviceId`                             | ID of the device being connected.                                |
| `Disconnect`          | `device_id`          | `DeviceId`                             | ID of the device being disconnected.                             |
| `DeviceSignaling`     | `signal`             | `serde_json::Value`                    | Signal data for the device.                                      |
| `UserSignaling`       | `device_id`          | `DeviceId`                             | ID of the target device.                                         |
|                       | `signal`             | `serde_json::Value`                    | Signal data for the user.                                        |
</details>

<details>
<summary>UserCommand</summary>

These are commands send from the server to the user. They are mostly used for
keeping the app in sync with the state of the backend.

| Variant               | Parameter           | Type                                   | Description                                                      |
|-----------------------|---------------------|----------------------------------------|------------------------------------------------------------------|
| `UpdateDevices`       | `devices`            | `Vec<&'a Device>`                      | List of devices to update.                                       |
| `RemoveDevice`        | `device_id`          | `&'a DeviceId`                         | ID of the device to remove.                                      |
| `UpdateConnectedDevice` | `device`           | `&'a Device`                           | Updated details of the connected device.                         |
| `RemoveConnectedDevice` | `device_id`        | `&'a DeviceId`                         | ID of the connected device to remove.                            |
| `DeviceSignaling`     | `device_id`          | `&'a DeviceId`                         | ID of the device sending the signal.                             |
|                       | `signal`             | `serde_json::Value`                    | Signal data (e.g., RTCSessionDescription or RTCIceCandidate).    |
| `Error`               | `message`            | `String`                               | Error message string.                                            |
</details>

<details>
<summary>DeviceCommand</summary>

These commands need to be handled by the device.

| Variant               | Parameter           | Type                                   | Description                                                      |
|-----------------------|---------------------|----------------------------------------|------------------------------------------------------------------|
| `UserSignaling`       | `signal`             | `serde_json::Value`                    | Signal data from a user.                                         |
| `Error`               | `message`            | `String`                               | Error message string.                                            |
</details>


### Entites

#### Device 
Devices can be registered under 
[http://127.0.0.1:8080/signaling/register/device/{device_name}](http://127.0.0.1:8080/signaling/register/device/{device_name}), 
which opens a ws connection between the Device and the server. As soon as a
Device is registered, a User can `Connect` to the Device, which means that the 
Device is only available to that specific User.

#### User
The User is registered under 
[http://127.0.0.1:8080/signaling/register/user](http://127.0.0.1:8080/signaling/register/user), 
which opens a ws connection between the User and the server. Users are able to
`Connect` to devices, send `Request` to initiate the WebRTC connection 
process. 

## 💡 Resources
- [https://github.com/actix/examples/tree/master/websockets/chat-actorless](https://github.com/actix/examples/tree/master/websockets/chat-actorless)
- [https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API)

## 📖 License
This project is licensed under the GNU General Public License v3.0.  
See the [LICENSE](https://www.gnu.org/licenses/gpl-3.0.en.html) file for more details.

