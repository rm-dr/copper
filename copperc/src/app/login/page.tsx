"use client";
import { Button, PasswordInput, Text, TextInput } from "@mantine/core";
import styles from "./page.module.scss";
import { useState } from "react";
import Banner from "../../../public/banner.svg";
import { useForm } from "@mantine/form";
import { APIclient } from "../_util/api";

export default function Page() {
	let [loading, setLoading] = useState(false);
	let [error, setError] = useState<null | string>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			username: "",
			password: "",
		},
	});

	return (
		<main className={styles.main}>
			<form
				onSubmit={form.onSubmit((values) => {
					setLoading(true);
					setError(null);
					APIclient.POST("/auth/login", { body: values })
						.then(({ data, error }) => {
							if (error !== undefined) {
								setLoading(false);
								setError("Login failed");
							} else {
								// Middleware will redirect to main page
								location.reload();
							}
						})
						.catch((err) => {
							setError(`Login failed: ${err}`);
							setLoading(false);
						});
				})}
			>
				<div className={styles.login_div}>
					<Banner />
					<TextInput
						disabled={loading}
						placeholder="User"
						style={{ width: "100%" }}
						key={form.key("username")}
						{...form.getInputProps("username")}
					/>
					<PasswordInput
						disabled={loading}
						placeholder="Password"
						style={{ width: "100%" }}
						key={form.key("password")}
						{...form.getInputProps("password")}
					/>
					<Button color="red" type="submit" loading={loading} fullWidth>
						Sign in
					</Button>
					<Text c="red">{error}</Text>
				</div>
			</form>
		</main>
	);
}
