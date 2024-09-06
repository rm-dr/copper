import styles from "../page.module.scss";
import { Panel, PanelTitle } from "@/components/panel";
import { IconLock, IconPencil, IconUserEdit } from "@tabler/icons-react";
import { XIcon } from "@/components/icons";
import { useForm } from "@mantine/form";
import { Button, PasswordInput, Text, TextInput } from "@mantine/core";
import { useEffect, useState } from "react";
import { APIclient } from "@/lib/api";
import { UserInfoState, useUserInfoStore } from "@/lib/userinfo";

export function useInfoPanel(params: {}) {
	const user = useUserInfoStore((state) => state.user_info);

	return {
		node: (
			<>
				<Panel
					panel_id={styles.panel_id_info}
					icon={<XIcon icon={IconUserEdit} />}
					title={"User info"}
				>
					<InfoSectionBasic user={user} />
					<InfoSectionPassword />
				</Panel>
			</>
		),
	};
}

function InfoSectionBasic(params: { user: UserInfoState["user_info"] }) {
	const user = params.user;
	const set_info = useUserInfoStore((state) => state.set_info);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [email, setEmail] = useState(user?.email || "");

	useEffect(() => {
		setEmail(user?.email || "");
	}, [setEmail, user?.email]);

	return (
		<>
			<PanelTitle icon={<XIcon icon={IconPencil} />} title={"Basic info"} />
			<div className={styles.settings_container}>
				<div className={styles.label_row}>
					<div className={styles.label}>
						<Text c="dimmed">Username:</Text>
					</div>
					<div className={styles.item}>
						<TextInput
							placeholder="Set username"
							disabled={true}
							// || "" is required to avoid a react error
							value={user?.name || ""}
						/>
					</div>
				</div>

				<div className={styles.label_row}>
					<div className={styles.label}>
						<Text c="dimmed">Email:</Text>
					</div>
					<div className={styles.item}>
						<TextInput
							placeholder="email is unset"
							disabled={isLoading}
							value={email}
							onChange={(x) => setEmail(x.currentTarget.value)}
						/>
					</div>
				</div>

				<Button.Group>
					<Button
						variant="light"
						fullWidth
						color="red"
						onClick={() => {
							setEmail(user?.email || "");
							setErrorMessage(null);
						}}
						disabled={isLoading}
					>
						Reset
					</Button>
					<Button
						variant="filled"
						color="green"
						fullWidth
						type="submit"
						loading={isLoading}
						onClick={() => {
							setLoading(true);
							setErrorMessage(null);

							if (user === null) {
								return;
							}

							// Minimum delay so user sees loader
							Promise.all([
								new Promise((r) => setTimeout(r, 500)),
								APIclient.POST("/auth/user/set_info", {
									body: {
										user: user.id,
										color: { action: "Unchanged" },
										email:
											email === ""
												? { action: "Clear" }
												: { action: "Set", value: email },
									},
								}),
							])
								.then(([_, { data, error }]) => {
									if (error !== undefined) {
										throw error;
									}

									set_info({
										...user,
										email: email === "" ? null : email,
									});
									setLoading(false);
								})
								.catch((err) => {
									setLoading(false);
									setErrorMessage(`${err}`);
								});
						}}
					>
						Save
					</Button>
				</Button.Group>
				<Text c="red" ta="center">
					{errorMessage}
				</Text>
			</div>
		</>
	);
}

function InfoSectionPassword(params: {}) {
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm<{
		my_password: null | string;
		new_password: null | string;
		new_password_repeat: null | string;
	}>({
		mode: "uncontrolled",
		initialValues: {
			my_password: null,
			new_password: null,
			new_password_repeat: null,
		},
		validate: {
			my_password: (value) => {
				if (value === null) {
					return "This field is required";
				}

				return null;
			},

			new_password: (value) => {
				if (value === null) {
					return "This field is required";
				}

				if (value.trim().length === 0) {
					return "New password must not be empty";
				}

				return null;
			},

			new_password_repeat: (value) => {
				if (value === null || value.trim().length === 0) {
					return "This field is required";
				}

				return null;
			},
		},
	});

	const reset = () => {
		form.reset();
		setErrorMessage(null);
	};

	const user = useUserInfoStore((state) => state.user_info);

	return (
		<>
			<PanelTitle icon={<XIcon icon={IconLock} />} title={"Password"} />

			<form
				onSubmit={form.onSubmit((values) => {
					setLoading(true);
					setErrorMessage(null);

					if (values.new_password === null || values.my_password === null) {
						throw Error(
							"Entered unreachable code: form state is null, this should've been caught by `validate`",
						);
					}

					if (values.new_password !== values.new_password_repeat) {
						setLoading(false);
						setErrorMessage("Passwords do not match");
						return;
					}

					if (user === null) {
						return;
					}

					// Minimum delay so user sees loader
					Promise.all([
						new Promise((r) => setTimeout(r, 500)),
						APIclient.POST("/auth/user/set_password", {
							body: {
								user: user.id,
								new_password: values.new_password,
								my_password: values.my_password,
							},
						}),
					])
						.then(([_, { data, error }]) => {
							if (error !== undefined) {
								throw error;
							}
							reset();
							setLoading(false);
						})
						.catch((err) => {
							setLoading(false);
							setErrorMessage(`${err}`);
						});
				})}
			>
				<div className={styles.settings_container}>
					<div className={styles.label_row}>
						<div className={styles.label}>
							<Text c="dimmed">Current:</Text>
						</div>
						<div className={styles.item}>
							<PasswordInput
								data-autofocus
								placeholder="Enter your password"
								disabled={isLoading}
								key={form.key("my_password")}
								{...form.getInputProps("my_password")}
							/>
						</div>
					</div>

					<div className={styles.label_row}>
						<div className={styles.label}>
							<Text c="dimmed">New:</Text>
						</div>
						<div className={styles.item}>
							<PasswordInput
								placeholder="Enter new password"
								disabled={isLoading}
								key={form.key("new_password")}
								{...form.getInputProps("new_password")}
							/>
						</div>
					</div>

					<div className={styles.label_row}>
						<div className={styles.label}>
							<Text c="dimmed">Repeat:</Text>
						</div>
						<div className={styles.item}>
							<PasswordInput
								data-autofocus
								placeholder="Repeat new password"
								disabled={isLoading}
								key={form.key("new_password_repeat")}
								{...form.getInputProps("new_password_repeat")}
							/>
						</div>
					</div>

					<Button.Group>
						<Button
							variant="light"
							fullWidth
							color="red"
							onClick={reset}
							disabled={isLoading}
						>
							Reset
						</Button>
						<Button
							variant="filled"
							color="green"
							fullWidth
							type="submit"
							loading={isLoading}
						>
							Save
						</Button>
					</Button.Group>
					<Text c="red" ta="center">
						{errorMessage}
					</Text>
				</div>
			</form>
		</>
	);
}
