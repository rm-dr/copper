import { XIconTrash } from "@/app/components/icons";
import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { useForm } from "@mantine/form";
import { UserInfo } from "../_grouptree";
import { ModalBase } from "@/app/components/modal_base";

export function useDeleteUserModal(params: {
	user: UserInfo;
	onChange: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			user: "",
		},
		validate: {
			user: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				if (value !== params.user.name) {
					return "Username doesn't match";
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
				title="Delete attribute"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="red" size="sm">
						This action will irreversably delete a user.
					</Text>
					<Text c="red" size="sm">
						Enter
						<Text c="orange" span>{` ${params.user.name} `}</Text>
						below to confirm.
					</Text>
				</div>
				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						fetch("/api/auth/user/del", {
							method: "delete",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify({
								user: params.user.id,
							}),
						})
							.then((res) => {
								setLoading(false);
								if (!res.ok) {
									res.text().then((text) => {
										setErrorMessage(text);
									});
								} else {
									params.onChange();
									reset();
								}
							})
							.catch((err) => {
								setLoading(false);
								setErrorMessage(`Error: ${err}`);
							});
					})}
				>
					<TextInput
						data-autofocus
						placeholder="Enter username"
						disabled={isLoading}
						key={form.key("user")}
						{...form.getInputProps("user")}
					/>

					<Button.Group style={{ marginTop: "1rem" }}>
						<Button
							variant="light"
							fullWidth
							color="red"
							onMouseDown={reset}
							disabled={isLoading}
						>
							Cancel
						</Button>
						<Button
							variant="filled"
							color="red"
							fullWidth
							leftSection={<XIconTrash />}
							type="submit"
						>
							Confirm
						</Button>
					</Button.Group>

					<Text c="red" ta="center">
						{errorMessage}
					</Text>
				</form>
			</ModalBase>
		),
	};
}
