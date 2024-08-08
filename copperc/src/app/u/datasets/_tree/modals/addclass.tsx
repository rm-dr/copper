import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { ModalBase } from "@/app/components/modal_base";
import { useForm } from "@mantine/form";
import { XIcon } from "@/app/components/icons";
import { IconFolderPlus } from "@tabler/icons-react";
import { APIclient } from "@/app/_util/api";

export function useAddClassModal(params: {
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm<{
		name: null | string;
	}>({
		mode: "uncontrolled",
		initialValues: {
			name: null,
		},
		validate: {
			name: (value) => {
				if (value === null) {
					return "This field is required";
				}

				if (value.trim().length === 0) {
					return "Name must not be empty";
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
				title="Add a class"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						Add a class to the dataset
						<Text
							c="var(--mantine-primary-color-4)"
							span
						>{` ${params.dataset_name}`}</Text>
						:
					</Text>
				</div>
				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						if (values.name === null) {
							throw Error(
								"Entered unreachable code: name is null, this should've been caught by `validate`",
							);
						}

						APIclient.POST("/class/add", {
							body: {
								new_class_name: values.name,
								dataset: params.dataset_name,
							},
						})
							.then(({ data, error }) => {
								if (error !== undefined) {
									throw error;
								}

								setLoading(false);
								params.onSuccess();
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
						placeholder="New class name"
						disabled={isLoading}
						key={form.key("name")}
						{...form.getInputProps("name")}
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
							Create class
						</Button>
					</Button.Group>
					<Text c="red" ta="center">
						{errorMessage ? errorMessage : null}
					</Text>
				</form>
			</ModalBase>
		),
	};
}
