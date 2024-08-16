"use client";

import "@mantine/core/styles.css";
import { MantineProvider, createTheme } from "@mantine/core";

import { useEffect } from "react";
import { APIclient } from "./_util/api";
import { useUserInfoStore } from "./_util/userinfo";

export default function Provider({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	const setinfo = useUserInfoStore((state) => state.set_info);
	useEffect(() => {
		APIclient.GET("/auth/me")
			.then(({ data, error }) => {
				if (error !== undefined) {
					throw error;
				}
				setinfo(data);
			})
			.catch((e) => {
				console.error(e);
			});
	}, [setinfo]);

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
