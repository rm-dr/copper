"use client";

import "@mantine/core/styles.css";
import { MantineProvider, createTheme } from "@mantine/core";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";

const theme = createTheme({
	fontFamily: GeistSans.style.fontFamily,
	fontFamilyMonospace: GeistMono.style.fontFamily,
	primaryColor: "blue",
});

export default function Provider({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<>
			<MantineProvider theme={theme} forceColorScheme="dark">
				{children}
			</MantineProvider>
		</>
	);
}
