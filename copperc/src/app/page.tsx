"use client";
import { Button, PasswordInput, Text, TextInput } from "@mantine/core";
import styles from "./page.module.scss";
import Banner from "../../public/banner.svg";
import { useForm } from "@mantine/form";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";
import { useEffect } from "react";

export default function Page() {
	useEffect(() => {
		edgeclient.GET("/user/me").then(({ response }) => {
			if (response.status === 200) {
				location.replace("/u");
			}
		});
	}, []);

	const doLogin = useMutation({
		mutationFn: async (body: components["schemas"]["LoginRequest"]) => {
			return await edgeclient.POST("/login", {
				body,
			});
		},

		onSuccess: ({ response }) => {
			if (response.status !== 200) {
				throw new Error("could not log in");
			} else {
				location.replace("/u");
			}
		},

		onError: (err) => {
			throw err;
		},
	});

	const form = useForm<{
		email: null | string;
		password: null | string;
	}>({
		mode: "uncontrolled",
		initialValues: {
			email: null,
			password: null,
		},
		validate: {
			email: (value) => {
				if (value === null || value.trim().length === 0) {
					return "email is required";
				}
				return null;
			},

			password: (value) => {
				if (value === null || value.trim().length === 0) {
					return "password is required";
				}
				return null;
			},
		},
	});

	return (
		<main className={styles.main}>
			<form
				onSubmit={form.onSubmit((values) => {
					const email = values.email;
					const password = values.password;
					if (email === null || password === null) {
						// Not possible, caught by validator.
						return;
					}

					doLogin.mutate({
						email,
						password,
					});
				})}
			>
				<div className={styles.login_div}>
					<Banner />

					<TextInput
						disabled={doLogin.isPending}
						placeholder="Email"
						style={{ width: "100%" }}
						key={form.key("email")}
						{...form.getInputProps("email")}
					/>

					<PasswordInput
						disabled={doLogin.isPending}
						placeholder="Password"
						style={{ width: "100%" }}
						key={form.key("password")}
						{...form.getInputProps("password")}
					/>

					<Button
						c="primary"
						type="submit"
						loading={doLogin.isPending}
						fullWidth
					>
						Sign in
					</Button>

					{doLogin.error === null ? null : (
						<Text c="red" ta="center">
							{doLogin.error.message}
						</Text>
					)}
				</div>
			</form>
		</main>
	);
}
