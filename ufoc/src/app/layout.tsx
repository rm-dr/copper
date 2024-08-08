import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.scss";

// Mantine setup
import Provider from "./provider";
import { ColorSchemeScript } from "@mantine/core";

const inter = Inter({ subsets: ["latin"] });

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
			<body className={inter.className}>
				<Provider>{children}</Provider>
			</body>
		</html>
	);
}
