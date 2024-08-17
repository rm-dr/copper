import styles from "../page.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";
import { IconLock, IconPencil, IconUserEdit } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { useForm } from "@mantine/form";
import { Button, PasswordInput, Text, TextInput } from "@mantine/core";
import { useState } from "react";
import { APIclient } from "@/app/_util/api";
import { useUserInfoStore } from "@/app/_util/userinfo";

export function useInfoPanel(params: {}) {
	return {
		node: (
			<>
				<Panel
					panel_id={styles.panel_id_info}
					icon={<XIcon icon={IconUserEdit} />}
					title={"User info"}
				>
					<InfoSectionBasic />
					<InfoSectionPassword />
				</Panel>
			</>
		),
	};
}

function InfoSectionBasic(params: {}) {
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const user = useUserInfoStore((state) => state.user_info);
	const set_info = useUserInfoStore((state) => state.set_info);

	const form = useForm<{
		email: string | null;
	}>({
		mode: "uncontrolled",
		initialValues: {
			email: user?.email || null,
		},
		validate: {
			email: (value) => {
				// empty string email => remove from account
				if (value === null) {
					return "Email must not be null";
				}

				return null;
			},
		},
	});

	const reset = () => {
		form.reset();
		setErrorMessage(null);
	};

	return (
		<>
			<PanelTitle icon={<XIcon icon={IconPencil} />} title={"Basic info"} />
			<form
				onSubmit={form.onSubmit((values) => {
					setLoading(true);
					setErrorMessage(null);

					if (values.email === null) {
						throw Error(
							"Entered unreachable code: form state is null, this should've been caught by `validate`",
						);
					}

					if (user === null) {
						return;
					}

					APIclient.POST("/auth/user/set_info", {
						body: {
							user: user.id,
							color: { action: "Unchanged" },
							email:
								values.email === ""
									? { action: "Clear" }
									: { action: "Set", value: values.email },
						},
					})
						.then(({ data, error }) => {
							if (error !== undefined) {
								throw error;
							}
							//reset();
							set_info({
								...user,
								email: values.email,
							});
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
							<Text c="dimmed">Username:</Text>
						</div>
						<div className={styles.item}>
							<TextInput
								placeholder="Set username"
								disabled={true}
								value={user?.name}
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
								key={form.key("email")}
								disabled={isLoading}
								{...form.getInputProps("email")}
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

					APIclient.POST("/auth/user/set_password", {
						body: {
							user: user.id,
							new_password: values.new_password,
							my_password: values.my_password,
						},
					})
						.then(({ data, error }) => {
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
							<Text c="dimmed" size="sm">
								New password:
							</Text>
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
