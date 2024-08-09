import { Button, PasswordInput, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { ModalBase } from "@/app/components/modal_base";
import { useForm } from "@mantine/form";
import { XIcon } from "@/app/components/icons";
import { IconTrash } from "@tabler/icons-react";
import { APIclient } from "@/app/_util/api";
import { components } from "@/app/_util/api/openapi";

export function useSetPasswordModal(params: {
	user: components["schemas"]["UserInfo"];
	onChange: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);
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
		setLoading(false);
		setErrorMessage(null);
		close();
	};

	return {
		open,
		modal: (
			<ModalBase
				opened={opened}
				close={reset}
				title="Change user password"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						You are changing
						<Text
							c="var(--mantine-primary-color-4)"
							span
						>{` ${params.user.name}'s `}</Text>
						password.
					</Text>
				</div>
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

						APIclient.POST("/auth/user/set_password", {
							body: {
								user: params.user.id,
								new_password: values.new_password,
								my_password: values.my_password,
							},
						})
							.then(({ data, error }) => {
								if (error !== undefined) {
									throw error;
								}

								setLoading(false);
								params.onChange();
								reset();
							})
							.catch((err) => {
								setLoading(false);
								setErrorMessage(`${err}`);
							});
					})}
				>
					<div
						style={{
							display: "flex",
							flexDirection: "column",
							gap: "0.5rem",
						}}
					>
						<PasswordInput
							data-autofocus
							placeholder="Enter your password"
							disabled={isLoading}
							key={form.key("my_password")}
							{...form.getInputProps("my_password")}
						/>

						<PasswordInput
							placeholder="Enter new password"
							disabled={isLoading}
							key={form.key("new_password")}
							{...form.getInputProps("new_password")}
						/>

						<PasswordInput
							data-autofocus
							placeholder="Repeat new password"
							disabled={isLoading}
							key={form.key("new_password_repeat")}
							{...form.getInputProps("new_password_repeat")}
						/>

						<Button.Group style={{ marginTop: "1rem" }}>
							<Button
								variant="light"
								fullWidth
								color="red"
								onClick={reset}
								disabled={isLoading}
							>
								Cancel
							</Button>
							<Button
								variant="filled"
								color="green"
								fullWidth
								leftSection={<XIcon icon={IconTrash} />}
								type="submit"
								loading={isLoading}
							>
								Confirm
							</Button>
						</Button.Group>
					</div>
					<Text c="red" ta="center">
						{errorMessage}
					</Text>
				</form>
			</ModalBase>
		),
	};
}
