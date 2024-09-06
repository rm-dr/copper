"use client";

import "@mantine/core/styles.css";
import { MantineProvider } from "@mantine/core";
import { useUserInfoStore } from "../lib/userinfo";
import { useEffect } from "react";
import { APIclient } from "../lib/api";

export default function Provider({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	const set_info = useUserInfoStore((state) => state.set_info);

	useEffect(() => {
		APIclient.GET("/auth/me")
			.then(({ data, error }) => {
				if (error !== undefined) {
					throw error;
				}
				set_info(data);
			})
			.catch((err) => {
				console.error(err);
			});
	}, [set_info]);

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
