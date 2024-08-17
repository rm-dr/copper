"use client";

import "@mantine/core/styles.css";
import { MantineProvider } from "@mantine/core";
import { useUserInfoStore } from "./_util/userinfo";

export default function Provider({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<>
			<MantineProvider
				theme={useUserInfoStore((state) => state.theme)}
				forceColorScheme="dark"
			>
				{children}
			</MantineProvider>
		</>
	);
}
