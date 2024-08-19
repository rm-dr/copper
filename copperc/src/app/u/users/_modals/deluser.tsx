import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { useForm } from "@mantine/form";
import { ModalBase } from "@/app/components/modal_base";
import { XIcon } from "@/app/components/icons";
import { IconTrash } from "@tabler/icons-react";
import { APIclient } from "@/app/_util/api";
import { components } from "@/app/_util/api/openapi";

export function useDeleteUserModal(params: {
	user: components["schemas"]["UserInfo"];
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

						APIclient.DELETE("/auth/user/del", {
							body: {
								user: params.user.id,
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
								setErrorMessage(err);
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
							onClick={reset}
							disabled={isLoading}
						>
							Cancel
						</Button>
						<Button
							variant="filled"
							color="red"
							fullWidth
							leftSection={<XIcon icon={IconTrash} />}
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
