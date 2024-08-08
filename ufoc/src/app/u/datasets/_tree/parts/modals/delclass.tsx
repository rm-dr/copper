import { XIconTrash } from "@/app/components/icons";
import { Button, Text, TextInput } from "@mantine/core";
import { useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { ModalBase } from "./modal_base";
import { useForm } from "@mantine/form";

export function useDeleteClassModal(params: {
	dataset_name: string;
	class_name: string;
	onSuccess: () => void;
}) {
	const [isLoading, setLoading] = useState(false);
	const [opened, { open, close }] = useDisclosure(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			class: "",
			dataset: params.dataset_name,
		},
		validate: {
			class: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				if (value !== params.class_name) {
					return "Class name doesn't match";
				}

				return null;
			},

			dataset: (value) => {
				if (value !== params.dataset_name) {
					return "Dataset name doesn't match";
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
				title="Delete class"
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
						<Text c="orange" span>{` ${params.class_name} `}</Text>
						below to confirm.
					</Text>
				</div>
				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						fetch("/api/class/del", {
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
						placeholder="Enter class name"
						size="sm"
						disabled={isLoading}
						key={form.key("class")}
						{...form.getInputProps("class")}
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
							fullWidth
							color="red"
							loading={isLoading}
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
