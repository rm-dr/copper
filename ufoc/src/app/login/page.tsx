"use client";
import { cookies } from "next/headers";
import { Button, PasswordInput, Text, TextInput } from "@mantine/core";
import styles from "./page.module.scss";
import { useState } from "react";

import Banner from "../../../public/banner.svg";

export default function Page() {
	let [user, setUser] = useState("");
	let [password, setPassword] = useState("");
	let [error, setError] = useState<null | string>(null);
	let [loading, setLoading] = useState(false);

	return (
		<main className={styles.main}>
			<div className={styles.login_div}>
				<Banner />
				<TextInput
					onChange={(e) => {
						setError(null);
						setUser(e.currentTarget.value);
					}}
					error={error !== null}
					disabled={loading}
					placeholder="User"
					style={{ width: "100%" }}
				/>
				<PasswordInput
					onChange={(e) => {
						setError(null);
						setPassword(e.currentTarget.value);
					}}
					error={error !== null}
					disabled={loading}
					placeholder="Password"
					style={{ width: "100%" }}
				/>
				<Button
					color={error === null ? undefined : "red"}
					onClick={() => {
						setLoading(true);
						fetch("/api/auth/login", {
							method: "post",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify({
								username: user,
								password,
							}),
						})
							.then((res) => {
								if (res.status === 400) {
									setLoading(false);
									setError("Invalid username or password");
								} else {
									return res.text().then((text) => {
										setLoading(false);

										// Middleware will redirect to main page
										location.reload();
									});
								}
							})
							.catch((err) => {
								setLoading(false);
								setError(err.text);
							});
					}}
					loading={loading}
					fullWidth
				>
					Sign in
				</Button>
				<Text c="red">{error}</Text>
			</div>
		</main>
	);
}
