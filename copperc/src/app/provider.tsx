"use client";

import "@mantine/core/styles.css";
import "@mantine/dropzone/styles.css";
import { createTheme, MantineProvider, virtualColor } from "@mantine/core";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NavigationBlockerProvider } from "@/components/navblock";

const queryClient = new QueryClient();

const theme = createTheme({
	primaryColor: "grape",
	colors: {
		active: virtualColor({
			name: "active",
			dark: "yellow",
			light: "yellow",
		}),
	},
	fontFamily: GeistSans.style.fontFamily,
	fontFamilyMonospace: GeistMono.style.fontFamily,
});

export default function Provider({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<>
			<NavigationBlockerProvider>
				<QueryClientProvider client={queryClient}>
					<MantineProvider theme={theme} forceColorScheme="dark">
						{children}
					</MantineProvider>
				</QueryClientProvider>
			</NavigationBlockerProvider>
		</>
	);
}
