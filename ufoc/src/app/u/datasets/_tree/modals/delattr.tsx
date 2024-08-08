import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { ModalBase } from "@/app/components/modal_base";
import { useForm } from "@mantine/form";
import { XIcon } from "@/app/components/icons";
import { IconTrash } from "@tabler/icons-react";

export function useDeleteAttrModal(params: {
	dataset_name: string;
	class_name: string;
	attr_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			class: params.class_name,
			dataset: params.dataset_name,
			attr: "",
		},
		validate: {
			attr: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				if (value !== params.attr_name) {
					return "Attribute name doesn't match";
				}

				return null;
			},

			class: (value) => {
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
				title="Delete attribute"
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
						<Text c="orange" span>{` ${params.attr_name} `}</Text>
						below to confirm.
					</Text>
				</div>
				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						fetch("/api/attr/del", {
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
						placeholder="Enter attribute name"
						disabled={isLoading}
						key={form.key("attr")}
						{...form.getInputProps("attr")}
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
