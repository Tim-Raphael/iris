import type { NextConfig } from "next";

const nextConfig: NextConfig = {
    env: {
        BASE_API_URL: "192.168.178.77:8080",
        // BASE_API_URL: "127.0.0.1:8080",
    },
};

export default nextConfig;
