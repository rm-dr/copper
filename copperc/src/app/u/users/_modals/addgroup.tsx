import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { useForm } from "@mantine/form";
import { ModalBase } from "@/components/modal_base";
import { XIcon } from "@/components/icons";
import { IconFolderPlus } from "@tabler/icons-react";
import { APIclient } from "@/lib/api";
import { components } from "@/lib/api/openapi";

export function useAddGroupModal(params: {
	group: components["schemas"]["GroupInfo"];
	onChange: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			name: "",
		},
		validate: {
			name: (value) =>
				value.trim().length === 0 ? "Group name cannot be empty" : null,
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
				title="Create a group"
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
						setLoading(true);
						setErrorMessage(null);

						APIclient.POST("/auth/group/add", {
							body: {
								parent: params.group?.id,
								name: values.name,
							},
						})

							.then(({ data, error }) => {
								if (error !== undefined) {
									throw error;
								}

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
						placeholder="group name"
						disabled={isLoading}
						key={form.key("name")}
						{...form.getInputProps("name")}
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
							leftSection={<XIcon icon={IconFolderPlus} />}
							type="submit"
						>
							Create group
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
