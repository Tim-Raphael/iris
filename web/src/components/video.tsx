"use client";

import { useRef, useEffect } from "react";

function startRendering(
    inputCanvas: HTMLCanvasElement,
    outputCanvas: HTMLCanvasElement,
    inputCanvasContext: CanvasRenderingContext2D,
    outputCanvasContext: CanvasRenderingContext2D,
    video: HTMLVideoElement
): () => void {
    let animationFrameId: number;
    const videoPromise = video.play();

    function handleResize(): void {
        const height = outputCanvas.offsetHeight;
        const width = outputCanvas.offsetWidth;

        if (width === 0 || height === 0) return;

        inputCanvas.height = height;
        inputCanvas.width = width;
        outputCanvas.height = height;
        outputCanvas.width = width;
    }

    function renderFrame(): void {
        const height = outputCanvas.offsetHeight;
        const width = outputCanvas.offsetWidth;

        if (width === 0 || height === 0) return;

        inputCanvasContext.drawImage(video, 0, 0, width, height);
        const imageData = inputCanvasContext.getImageData(0, 0, width, height);
        outputCanvasContext.putImageData(imageData, 0, 0);
        animationFrameId = requestAnimationFrame(renderFrame);
    }

    window.addEventListener("resize", handleResize);

    handleResize();
    renderFrame();

    return () => {
        videoPromise.then(() => video.pause());
        window.removeEventListener("resize", handleResize);
        cancelAnimationFrame(animationFrameId);
    }
}

export default function Video({ remoteStream }: { remoteStream: MediaStream | null }) {
    const inputCanvasRef = useRef<HTMLCanvasElement | null>(null);
    const outputCanvasRef = useRef<HTMLCanvasElement | null>(null);
    const videoRef = useRef<HTMLVideoElement | null>(null);
    const placeholderVideoRef = useRef<HTMLVideoElement | null>(null);

    useEffect(() => {
        if (!videoRef.current || !inputCanvasRef.current || !outputCanvasRef.current || !placeholderVideoRef.current)
            throw new Error("Missing a Element here. Could not render the stream!");

        videoRef.current.srcObject = remoteStream;

        const video = remoteStream ? videoRef.current : placeholderVideoRef.current;
        const inputCanvas = inputCanvasRef.current;
        const outputCanvas = outputCanvasRef.current;
        const outputCanvasContext = outputCanvas.getContext("2d");
        const inputCanvasContext = inputCanvas.getContext("2d", {
            willReadFrequently: true,
        });

        if (!outputCanvasContext || !inputCanvasContext)
            throw new Error("Could not get Canvas-Context!");

        const stopRendering = startRendering(
            inputCanvas,
            outputCanvas,
            inputCanvasContext,
            outputCanvasContext,
            video,
        );

        return () => {
            stopRendering();
        };
    }, [inputCanvasRef, outputCanvasRef, videoRef, placeholderVideoRef, remoteStream]);

    return (
        <div className="bg-muted/50 rounded border">
            <video
                ref={videoRef}
                playsInline
                className="hidden"
            ></video>
            <video
                ref={placeholderVideoRef}
                src="/video/placeholder_animation_iris.mp4"
                playsInline
                loop
                muted
                className="hidden"
            ></video>
            <div className="flex">
                <canvas ref={inputCanvasRef} className="hidden"></canvas>
                <canvas
                    ref={outputCanvasRef}
                    className="aspect-video w-full"
                ></canvas>
            </div>
        </div >
    );
}
