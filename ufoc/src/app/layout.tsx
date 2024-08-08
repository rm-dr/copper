import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.scss";
import NavBar from "./components/navbar";
import SideBar from "./components/sidebar";
import styles from "./layout.module.scss";

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
				<Provider>
					<div className={styles.navbarContainer}>
						<NavBar />
					</div>
					<div className={styles.lowercontent}>
						<div className={styles.sidebarContainer}>
							<SideBar />
						</div>
						<div className={styles.contentContainer}>{children}</div>
					</div>
				</Provider>
			</body>
		</html>
	);
}
