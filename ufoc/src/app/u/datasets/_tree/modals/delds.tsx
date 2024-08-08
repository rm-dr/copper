import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { useState } from "react";
import { ModalBase } from "@/app/components/modal_base";
import { XIcon } from "@/app/components/icons";
import { IconTrash } from "@tabler/icons-react";

export function useDeleteDatasetModal(params: {
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [isLoading, setLoading] = useState(false);
	const [opened, { open, close }] = useDisclosure(false);
	const [errorMessage, setErrorMessage] = useState<null | string>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			dataset_name: "",
		},
		validate: {
			dataset_name: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				if (value !== params.dataset_name) {
					return "Dataset name doesn't match";
				}

				return null;
			},
		},
	});

	const reset = () => {
		setLoading(false);
		form.reset();
		setErrorMessage(null);
		close();
	};

	return {
		open,
		modal: (
			<ModalBase
				opened={opened}
				close={reset}
				title="Delete dataset"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="red" size="sm">
						This action will irreversably destroy data.
					</Text>
					<Text c="red" size="sm">
						Enter
						<Text c="orange" span>{` ${params.dataset_name} `}</Text>
						below to confirm.
					</Text>
				</div>

				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						fetch("/api/dataset/del", {
							method: "delete",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify(values),
						})
							.then((res) => {
								setLoading(false);
								if (!res.ok) {
									res.text().then((text) => {
										setErrorMessage(text);
									});
								} else {
									params.onSuccess();
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
						placeholder="Enter dataset name"
						disabled={isLoading}
						key={form.key("dataset_name")}
						{...form.getInputProps("dataset_name")}
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
							leftSection={<XIcon icon={IconTrash} />}
							color="red"
							loading={isLoading}
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
