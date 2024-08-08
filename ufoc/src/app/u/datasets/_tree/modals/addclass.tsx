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

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			new_class_name: "",
			dataset: params.dataset_name,
		},
		validate: {
			new_class_name: (value) =>
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

						APIclient.POST("/class/add", { body: values })
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
						key={form.key("new_class_name")}
						{...form.getInputProps("new_class_name")}
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
