"use client";

import style from "./navbar.module.scss";
import Banner from "../../../../public/banner.svg";
import { Button, Menu, PasswordInput, Text, TextInput } from "@mantine/core";
import { edgeclient } from "@/lib/api/client";
import { useUserInfoQuery } from "@/lib/query";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { components } from "@/lib/api/openapi";
import { ModalBaseSmall, modalStyle } from "@/components/modalbase";
import { Book, Lock, UserCircle, UserPen } from "lucide-react";

export default function TopBar() {
	// Get user info, log out if not logged in.
	// Since the topbar is always shown, this `might` be enough to always log us out.
	// Warrants further investigation.
	const userInfo = useUserInfoQuery();

	const { open: openUpdate, modal: updateModal } = useUpdateModal({
		onSuccess: () => {
			location.reload();
		},
	});

	return (
		<>
			{updateModal}
			<div className={style.navbar}>
				<div className={style.banner}>
					<Banner />
				</div>

				<div className={style.usermenu}>
					<Menu trigger="click-hover" shadow="md">
						<Menu.Target>
							{userInfo.isLoading ? (
								<div className={style.usercontainer}>
									<Text c="dimmed">Loading...</Text>
								</div>
							) : (
								<div className={style.usercontainer}>
									<UserCircle />
									<Text>{userInfo.data?.data?.name}</Text>
								</div>
							)}
						</Menu.Target>

						<Menu.Dropdown>
							<Menu.Item
								leftSection={<UserPen size="1.3rem" />}
								onClick={() => {
									const info = userInfo.data?.data;
									if (info !== undefined) {
										openUpdate(info);
									}
								}}
							>
								Profile
							</Menu.Item>
							<Menu.Item
								leftSection={<Lock size="1.3rem" />}
								onClick={() => {
									edgeclient.POST("/logout").then(() => {
										window.location.replace("/");
									});
								}}
							>
								Log out
							</Menu.Item>

							<Menu.Divider />

							<Menu.Item leftSection={<Book size="1.3rem" />}>
								Documentation
							</Menu.Item>
						</Menu.Dropdown>
					</Menu>
				</div>
			</div>
		</>
	);
}

function useUpdateModal(params: { onSuccess: () => void }) {
	const [opened, { open, close }] = useDisclosure(false);
	const [errorMessage, setErrorMessage] = useState<null | string>(null);
	const [userInfo, setUserInfo] = useState<
		null | components["schemas"]["UserInfo"]
	>(null);

	const form = useForm<{
		email: null | string;
		name: null | string;
		password: null | string;
		password_again: null | string;
	}>({
		mode: "uncontrolled",
		initialValues: {
			name: null,
			email: null,
			password: null,
			password_again: null,
		},
		validate: {
			email: (value) => {
				if (value === null || value.trim().length === 0) {
					return "email is required";
				}
				return null;
			},

			name: (value) => {
				if (value === null || value.trim().length === 0) {
					return "this field is required";
				}
				return null;
			},
		},
	});

	const reset = () => {
		form.reset();
		setErrorMessage(null);
		close();
	};

	const doUpdate = useMutation({
		mutationFn: async (body: components["schemas"]["UpdateUserRequest"]) => {
			if (userInfo === null) {
				return null;
			}

			return await edgeclient.PATCH("/user/{user_id}", {
				params: { path: { user_id: userInfo?.id } },
				body,
			});
		},

		onSuccess: async (res) => {
			if (res === null) {
				return;
			}

			if (res.response.status === 200) {
				reset();
				params.onSuccess();
			}

			throw new Error(res.error);
		},

		onError: (err) => {
			throw err;
		},
	});

	return {
		open: (userInfo: components["schemas"]["UserInfo"]) => {
			setUserInfo(userInfo);
			form.setValues({
				email: userInfo.email,
				name: userInfo.name,
				password: null,
				password_again: null,
			});
			open();
		},
		modal: (
			<ModalBaseSmall
				hardtoclose
				opened={opened}
				close={() => {
					reset();
					close();
				}}
				title="Edit profile"
				keepOpen={doUpdate.isPending}
			>
				<form
					onSubmit={form.onSubmit((values) => {
						const email = values.email;
						const name = values.name;
						if (email === null || name === null) {
							// Not possible, caught by validator.
							return;
						}

						if (
							values.password !== null &&
							values.password !== values.password_again
						) {
							setErrorMessage("Passwords do not match!");
							return;
						}

						setErrorMessage(null);
						doUpdate.mutate({
							new_email: email,
							new_password: values.password,
							new_name: name,
						});
					})}
				>
					<div className={modalStyle.modal_outer_container}>
						<div className={modalStyle.modal_input_container}>
							<TextInput
								data-autofocus
								label="Name"
								placeholder="Ivan Kovalev"
								disabled={doUpdate.isPending}
								key={form.key("name")}
								{...form.getInputProps("name")}
							/>

							<TextInput
								label="Email"
								placeholder="ivank@betalupi.com"
								disabled={doUpdate.isPending}
								key={form.key("email")}
								{...form.getInputProps("email")}
							/>

							<PasswordInput
								label="Change password"
								disabled={doUpdate.isPending}
								key={form.key("password")}
								{...form.getInputProps("password")}
							/>

							<PasswordInput
								label="Repeat password"
								disabled={doUpdate.isPending}
								key={form.key("password_again")}
								{...form.getInputProps("password_again")}
							/>

							{errorMessage === null ? null : (
								<Text c="red" ta="center">
									{errorMessage}
								</Text>
							)}
						</div>

						<Button.Group style={{ width: "100%" }}>
							<Button
								variant="light"
								fullWidth
								color="red"
								onClick={reset}
								disabled={doUpdate.isPending}
							>
								Cancel
							</Button>
							<Button
								variant="filled"
								fullWidth
								c="primary"
								loading={doUpdate.isPending}
								type="submit"
							>
								Update profile
							</Button>
						</Button.Group>

						{doUpdate.error ? (
							<Text c="red" ta="center">
								{doUpdate.error.message}
							</Text>
						) : null}
					</div>
				</form>
			</ModalBaseSmall>
		),
	};
}
