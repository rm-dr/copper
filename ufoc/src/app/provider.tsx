"use client";

import "@mantine/core/styles.css";
import { MantineProvider, Modal, createTheme } from "@mantine/core";

const theme = createTheme({
	fontFamily: "Open Sans, sans-serif",
	primaryColor: "cyan",

	components: {
		Modal: Modal.extend({
			styles: {
				header: {
					backgroundColor: "var(--mantine-color-gray-8)",
				},
				content: {
					backgroundColor: "var(--mantine-color-gray-9)",
				},
			},
		}),
	},
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
