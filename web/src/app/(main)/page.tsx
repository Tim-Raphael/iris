"use client";

import {
    useEffect,
    useRef,
    useState,
    RefObject,
    Dispatch,
    SetStateAction
} from "react";

import Video from "~/src/components/video";
import Header from "~/src/components/header";

import { ServerCommand } from "~/src/lib/commands";
import { useIris } from "~/src/contexts/iris-context";

async function startConnection(
    activeDeviceId: string,
    peerConnection: RTCPeerConnection,
    sendCommand: (commnand: ServerCommand) => void,
    ondevicesignaling: RefObject<(signal: RTCSessionDescription | RTCIceCandidate) => void>,
    setRemoteStream: Dispatch<SetStateAction<MediaStream | null>>,
): Promise<void> {
    function handleDeviceSignal(signal: RTCSessionDescription | RTCIceCandidate): void {
        if ("sdp" in signal)
            peerConnection.setRemoteDescription(
                new RTCSessionDescription(signal),
            );
        else if ("candidate" in signal)
            peerConnection.addIceCandidate(signal);
    }

    function handleIceCandidate(event: RTCPeerConnectionIceEvent): void {
        if (!event.candidate) return;
        sendCommand({
            type: "UserSignaling",
            device_id: activeDeviceId,
            signal: event.candidate,
        });
    }

    function handleOntrack(event: RTCTrackEvent): void {
        const [stream] = event.streams;
        setRemoteStream(stream);
    }

    ondevicesignaling.current = handleDeviceSignal;
    peerConnection.onicecandidate = handleIceCandidate;
    peerConnection.ontrack = handleOntrack;

    peerConnection.createDataChannel("stream");

    const offer = await peerConnection.createOffer({
        offerToReceiveVideo: true,
    });

    await peerConnection.setLocalDescription(offer);

    if (!peerConnection.localDescription) return;

    sendCommand({
        type: "UserSignaling",
        device_id: activeDeviceId,
        signal: peerConnection.localDescription,
    });
}

export default function Home() {
    const { sendCommand, ondevicesignaling, activeDevice } = useIris();
    const peerConnection = useRef<RTCPeerConnection | null>(null);
    const [remoteStream, setRemoteStream] = useState<MediaStream | null>(null);

    useEffect(() => {
        if (!activeDevice) return;

        peerConnection.current = new RTCPeerConnection({
            iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
        });

        startConnection(
            activeDevice.id,
            peerConnection.current,
            sendCommand,
            ondevicesignaling,
            setRemoteStream,
        );

        return () => {
            if (!peerConnection.current) return;
            peerConnection.current.close();
            peerConnection.current = null;
            setRemoteStream(null);
        };
    }, [activeDevice]);

    return (
        <>
            <Header title={"Controls"} />
            <main className="flex flex-1 flex-col gap-4 p-4 pt-0">
                <Video remoteStream={remoteStream} />
                <div className="grid auto-rows-min gap-4 md:grid-cols-3">
                    <div className="aspect-video rounded-xl bg-muted/50" />
                    <div className="aspect-video rounded-xl bg-muted/50" />
                    <div className="aspect-video rounded-xl bg-muted/50" />
                </div>
            </main>
        </>
    );
}
