import { Button, PasswordInput, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { useForm } from "@mantine/form";
import { ModalBase } from "@/components/modal_base";
import { IconUserPlus } from "@tabler/icons-react";
import { XIcon } from "@/components/icons";
import { APIclient } from "@/lib/api";
import { components } from "@/lib/api/openapi";

export function useAddUserModal(params: {
	group?: components["schemas"]["GroupInfo"];
	onChange: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			username: "",
			password: "",
			passwordconfirm: "",
			email: "",
		},
		validate: {
			username: (value) =>
				value.trim().length === 0 ? "Username cannot be empty" : null,
			password: (value) =>
				value.trim().length === 0 ? "Password cannot be empty" : null,
			passwordconfirm: (value) =>
				value.trim().length === 0 ? "Password cannot be empty" : null,
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
				title="Add a user"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						Add a user in the group
						<Text c="gray" span>{` ${params.group?.name}`}</Text>:
					</Text>
				</div>
				<form
					onSubmit={form.onSubmit((values) => {
						if (params.group === undefined) {
							return;
						}

						setLoading(true);
						setErrorMessage(null);

						if (values.password !== values.passwordconfirm) {
							setLoading(false);
							setErrorMessage("Passwords do not match");
						}

						APIclient.POST("/auth/user/add", {
							body: {
								group: params.group.id,
								username: values.username,
								password: values.password,
								email: values.email === "" ? null : values.email,
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
							.catch((e) => {
								setLoading(false);
								setErrorMessage(e);
							});
					})}
				>
					<TextInput
						data-autofocus
						placeholder="username"
						disabled={isLoading}
						key={form.key("username")}
						{...form.getInputProps("username")}
						style={{
							margin: "0.5rem",
						}}
					/>
					<TextInput
						data-autofocus
						placeholder="email (optional)"
						disabled={isLoading}
						key={form.key("email")}
						{...form.getInputProps("email")}
						style={{
							margin: "0.5rem",
						}}
					/>
					<PasswordInput
						data-autofocus
						placeholder="password"
						disabled={isLoading}
						key={form.key("password")}
						{...form.getInputProps("password")}
						style={{
							margin: "0.5rem",
						}}
					/>
					<PasswordInput
						data-autofocus
						placeholder="confirm password"
						disabled={isLoading}
						key={form.key("passwordconfirm")}
						{...form.getInputProps("passwordconfirm")}
						style={{
							margin: "0.5rem",
						}}
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
							fullWidth
							color={errorMessage === null ? "green" : "red"}
							loading={isLoading}
							leftSection={<XIcon icon={IconUserPlus} />}
							type="submit"
						>
							Create user
						</Button>
					</Button.Group>
					<Text c="red" ta="center">
						{errorMessage ? errorMessage : ""}
					</Text>
				</form>
			</ModalBase>
		),
	};
}
