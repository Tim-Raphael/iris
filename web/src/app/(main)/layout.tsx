import "./globals.css";

import { AppSidebar } from "~/src/components/app-sidebar";
import { SidebarInset, SidebarProvider } from "~/src/components/ui/sidebar";
import { IrisProvider } from "~/src/contexts/iris-context";

import { Metadata } from "next";

export const metadata: Metadata = {
    title: "Iris - Webview",
    description:
        "A App that is used to manage and connect to remote controled cars using WebRTC.",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en">
            <body>
                <IrisProvider path="/signaling/register/user">
                    <SidebarProvider>
                        <AppSidebar />
                        <SidebarInset>{children}</SidebarInset>
                    </SidebarProvider>
                </IrisProvider>
            </body>
        </html>
    );
}
