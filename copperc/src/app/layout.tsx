import type { Metadata } from "next";
import "./globals.scss";

import Provider from "./provider";
import { ColorSchemeScript } from "@mantine/core";

export const metadata: Metadata = {
	title: "Copper",
	description: "Copper web ui",
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
			<body>
				<Provider>{children}</Provider>
			</body>
		</html>
	);
}
