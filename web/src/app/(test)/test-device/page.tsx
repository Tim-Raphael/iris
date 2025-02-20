"use client";

import { useEffect } from "react";

async function handlePeerConnection(stream: MediaStream, ws: WebSocket) {
    const peerConnection = new RTCPeerConnection({
        iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
    });

    peerConnection.onicecandidate = event => {
        ws.send(
            JSON.stringify({
                type: "DeviceSignaling",
                signal: event.candidate,
            }),
        );
    };

    stream.getTracks().forEach(track => peerConnection.addTrack(track, stream));

    ws.onmessage = async event => {
        const cmd = JSON.parse(event.data);

        console.log(cmd);

        if (cmd.signal.type === "offer") {
            peerConnection.setRemoteDescription(cmd.signal);
            const answer = await peerConnection.createAnswer();
            await peerConnection.setLocalDescription(answer);
            ws.send(
                JSON.stringify({ type: "DeviceSignaling", signal: answer }),
            );
        } else if (cmd.signal.candidate) {
            try {
                await peerConnection.addIceCandidate(cmd.signal);
            } catch (err) {
                console.error(`${err}`);
            }
        }
    };
}

export default function Home() {
    useEffect(() => {
        if (!process.env.BASE_API_URL) {
            console.error(
                "BASE_API_URL is not defined in environment variables.",
            );
            return;
        }

        const ws = new WebSocket(
            `ws://${process.env.BASE_API_URL}/signaling/register/device/John Doe`,
        );

        ws.onopen = () => console.log("WebSocket connection opened.");
        ws.onerror = error => console.error("WebSocket error:", error);
        ws.onclose = () => console.log("WebSocket connection closed.");

        navigator.mediaDevices
            .getUserMedia({
                video: { width: { ideal: 1920 }, height: { ideal: 1080 } },
                audio: false,
            })
            .then(async stream => await handlePeerConnection(stream, ws))
            .catch(err => console.error("Error:", err));

        return () => {
            ws.close();
        };
    }, []);

    return <></>;
}
