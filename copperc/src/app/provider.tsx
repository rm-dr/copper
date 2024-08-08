"use client";

import "@mantine/core/styles.css";
import { MantineProvider, createTheme } from "@mantine/core";

const theme = createTheme({
	fontFamily: "Open Sans, sans-serif",
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
