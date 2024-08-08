import type { Metadata } from "next";
import "./globals.scss";
import { GeistSans } from "geist/font/sans";
//import { GeistMono } from "geist/font/mono";

// Mantine setup
import Provider from "./provider";
import { ColorSchemeScript } from "@mantine/core";

export const metadata: Metadata = {
	title: "Copper",
	description: "TODO",
};

export default function RootLayout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<html lang="en">
			<head>
				<ColorSchemeScript />
			</head>
			<body className={GeistSans.className}>
				<Provider>{children}</Provider>
			</body>
		</html>
	);
}
