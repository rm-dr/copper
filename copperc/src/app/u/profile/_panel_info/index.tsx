import styles from "../page.module.scss";
import { Panel, PanelTitle } from "@/app/components/panel";
import {
	IconColorPicker,
	IconLock,
	IconPaint,
	IconPencil,
	IconSchema,
	IconUserEdit,
} from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { useForm } from "@mantine/form";
import {
	Button,
	ColorPicker,
	PasswordInput,
	Text,
	TextInput,
} from "@mantine/core";
import { useState } from "react";
import { APIclient } from "@/app/_util/api";
import { useUserInfoStore } from "@/app/_util/userinfo";

export function useInfoPanel(params: {}) {
	const setcolor = useUserInfoStore((state) => state.set_color);
	const [isLoading, setLoading] = useState(false);

	const [infoErrorMessage, setInfoErrorMessage] = useState<string | null>(null);
	const info_form = useForm<{
		name: string;
		email: string;
	}>({
		mode: "uncontrolled",
		initialValues: {
			name: "admin",
			email: "admin@copper.org",
		},
		validate: {
			name: (value) => {
				if (value.trim().length === 0) {
					return "Name must not be empty";
				}

				return null;
			},
		},
	});
	const info_reset = () => {
		info_form.reset();
		setInfoErrorMessage(null);
	};

	const [passwordErrorMessage, setPasswordErrorMessage] = useState<
		string | null
	>(null);
	const password_form = useForm<{
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

	const password_reset = () => {
		password_form.reset();
		setPasswordErrorMessage(null);
	};

	const [colorErrorMessage, setColorErrorMessage] = useState<string | null>(
		null,
	);
	const [color, setColor] = useState("#ff4f0f");
	const color_reset = () => {
		setColor("#ff4f0f");
		setColorErrorMessage(null);
	};

	return {
		node: (
			<>
				<Panel
					panel_id={styles.panel_id_pipe}
					icon={<XIcon icon={IconUserEdit} />}
					title={"User info"}
				>
					<PanelTitle icon={<XIcon icon={IconPencil} />} title={"Basic info"} />
					<form>
						<div className={styles.settings_container}>
							<div className={styles.label_row}>
								<div className={styles.label}>
									<Text c="dimmed">Username:</Text>
								</div>
								<div className={styles.item}>
									<TextInput
										placeholder="Set username"
										disabled={isLoading}
										key={info_form.key("name")}
										{...info_form.getInputProps("name")}
									/>
								</div>
							</div>

							<div className={styles.label_row}>
								<div className={styles.label}>
									<Text c="dimmed">Email:</Text>
								</div>
								<div className={styles.item}>
									<TextInput
										placeholder="Set email"
										key={info_form.key("email")}
										disabled={isLoading}
										{...info_form.getInputProps("email")}
									/>
								</div>
							</div>

							<Button.Group>
								<Button
									variant="light"
									fullWidth
									color="red"
									onClick={info_reset}
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
								{infoErrorMessage}
							</Text>
						</div>
					</form>

					<PanelTitle icon={<XIcon icon={IconLock} />} title={"Password"} />

					<form
						onSubmit={password_form.onSubmit((values) => {
							setLoading(true);
							setPasswordErrorMessage(null);

							if (values.new_password === null || values.my_password === null) {
								throw Error(
									"Entered unreachable code: form state is null, this should've been caught by `validate`",
								);
							}

							if (values.new_password !== values.new_password_repeat) {
								setLoading(false);
								setPasswordErrorMessage("Passwords do not match");
								return;
							}

							APIclient.POST("/auth/user/set_password", {
								body: {
									user: { id: 2 },
									new_password: values.new_password,
									my_password: values.my_password,
								},
							})
								.then(({ data, error }) => {
									if (error !== undefined) {
										throw error;
									}

									setLoading(false);
								})
								.catch((err) => {
									setLoading(false);
									setPasswordErrorMessage(`${err}`);
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
										key={password_form.key("my_password")}
										{...password_form.getInputProps("my_password")}
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
										key={password_form.key("new_password")}
										{...password_form.getInputProps("new_password")}
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
										key={password_form.key("new_password_repeat")}
										{...password_form.getInputProps("new_password_repeat")}
									/>
								</div>
							</div>

							<Button.Group>
								<Button
									variant="light"
									fullWidth
									color="red"
									onClick={password_reset}
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
								{passwordErrorMessage}
							</Text>
						</div>
					</form>
				</Panel>

				<Panel
					panel_id={styles.panel_id_pipe}
					icon={<XIcon icon={IconPaint} />}
					title={"Interface"}
				>
					<PanelTitle
						icon={<XIcon icon={IconColorPicker} />}
						title={"Primary color"}
					/>

					<div className={styles.settings_container}>
						Pick a primary color for your UI.
						<ColorPicker
							fullWidth
							format="hex"
							swatches={[
								"#2e2e2e",
								"#868e96",
								"#fa5252",
								"#e64980",
								"#be4bdb",
								"#7950f2",
								"#4c6ef5",
								"#228be6",
								"#15aabf",
								"#12b886",
								"#40c057",
								"#82c91e",
								"#fab005",
								"#fd7e14",
							]}
							onChange={(x) => {
								setColor(x);
								setcolor(x);
							}}
						/>
						<Button.Group>
							<Button
								variant="light"
								fullWidth
								color="red"
								onClick={color_reset}
								disabled={isLoading}
							>
								Reset
							</Button>
							<Button
								variant="filled"
								color="green"
								fullWidth
								onClick={() => {
									setLoading(true);
									setColorErrorMessage(null);

									APIclient.POST("/auth/user/set_info", {
										body: {
											user: 1 as any,
											color: { action: "Set", color: color },
											email: { action: "Unchanged" },
										},
									})
										.then(({ data, error }) => {
											if (error !== undefined) {
												throw error;
											}

											setLoading(false);
											setcolor(color);
										})
										.catch((err) => {
											setLoading(false);
											setColorErrorMessage(err);
										});
								}}
								loading={isLoading}
							>
								Save
							</Button>
						</Button.Group>
						<Text c="red" ta="center">
							{colorErrorMessage}
						</Text>
					</div>
				</Panel>
			</>
		),
	};
}
