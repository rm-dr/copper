import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { ModalBase } from "@/app/components/modal_base";
import { useForm } from "@mantine/form";
import { XIcon } from "@/app/components/icons";
import { IconTrash } from "@tabler/icons-react";
import { APIclient } from "@/app/_util/api";
import { components } from "@/app/_util/api/openapi";

export function useRenameDatasetModal(params: {
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			new_name: "",
		},
		validate: {
			new_name: (value) => {
				if (value.trim().length === 0) {
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
				title="Rename dataset"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						You are renaming the dataset
						<Text
							c="var(--mantine-primary-color-4)"
							span
						>{` ${params.dataset_name}`}</Text>
						.
					</Text>
				</div>
				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						APIclient.POST("/dataset/rename", {
							body: {
								old_name: params.dataset_name,
								new_name: values.new_name,
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
							.catch((err) => {
								setLoading(false);
								setErrorMessage(err);
							});
					})}
				>
					<TextInput
						data-autofocus
						placeholder="Enter new name"
						disabled={isLoading}
						key={form.key("new_name")}
						{...form.getInputProps("new_name")}
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

					<Text c="red" ta="center">
						{errorMessage}
					</Text>
				</form>
			</ModalBase>
		),
	};
}
