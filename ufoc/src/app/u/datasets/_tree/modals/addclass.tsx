import { Button, Text, TextInput } from "@mantine/core";

import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { ModalBase } from "@/app/components/modal_base";
import { useForm } from "@mantine/form";
import { XIcon } from "@/app/components/icons";
import { IconFolderPlus } from "@tabler/icons-react";

export function useAddClassModal(params: {
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			class: "",
			dataset: params.dataset_name,
		},
		validate: {
			class: (value) =>
				value.trim().length === 0 ? "Name cannot be empty" : null,
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
						<Text c="gray" span>{` ${params.dataset_name}`}</Text>:
					</Text>
				</div>
				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						fetch(`/api/class/add`, {
							method: "POST",
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
							.catch((e) => {
								setLoading(false);
								setErrorMessage(`Error: ${e}`);
							});
					})}
				>
					<TextInput
						data-autofocus
						placeholder="New class name"
						disabled={isLoading}
						key={form.key("class")}
						{...form.getInputProps("class")}
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
						{errorMessage ? errorMessage : ""}
					</Text>
				</form>
			</ModalBase>
		),
	};
}
