import { Metadata } from "next";

export const metadata: Metadata = {
    title: "Iris - Webview",
    description:
        "A Web App that is used to manage and connect to remote controled cars using WebRTC.",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en">
            <body>{children}</body>
        </html>
    );
}
