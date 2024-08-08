import { Button, Select, Text, TextInput } from "@mantine/core";
import { useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { XIconDatabasePlus } from "@/app/components/icons";
import { useForm } from "@mantine/form";
import { ModalBase } from "./modal_base";
import { datasetTypes } from "@/app/_util/datasets";

export function useNewDsModal(onSuccess: () => void) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			name: "",
			params: {
				type: null,
			},
		},
		validate: {
			name: (value) =>
				value.trim().length === 0 ? "Name cannot be empty" : null,
			params: {
				type: (value) => (value === null ? "Type cannot be empty" : null),
			},
		},
	});

	const reset = () => {
		form.reset();
		setErrorMessage(null);
		setLoading(false);
		close();
	};

	return {
		open,
		modal: (
			<ModalBase
				opened={opened}
				close={reset}
				title="Create a new dataset"
				keepOpen={isLoading}
			>
				<form
					onSubmit={form.onSubmit((values) => {
						setLoading(true);
						setErrorMessage(null);

						fetch(`/api/dataset/add`, {
							method: "POST",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify(values),
						}).then((res) => {
							setLoading(false);
							if (!res.ok) {
								if (res.status == 401) {
									setErrorMessage("Not authorized");
								} else {
									res.text().then(setErrorMessage);
								}
							} else {
								// Successfully created new dataset
								onSuccess();
								reset();
							}
						});
					})}
				>
					<TextInput
						data-autofocus
						placeholder="enter dataset name"
						disabled={isLoading}
						key={form.key("name")}
						{...form.getInputProps("name")}
					/>
					<Select
						required={true}
						style={{ marginTop: "1rem" }}
						placeholder="select dataset type"
						data={datasetTypes.map((x) => {
							return x.pretty_name;
						})}
						disabled={isLoading}
						comboboxProps={{
							transitionProps: { transition: "fade-down", duration: 200 },
						}}
						clearable
						key={form.key("params.type")}
						{...form.getInputProps("params.type")}
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
							color="green"
							loading={isLoading}
							leftSection={<XIconDatabasePlus />}
							type="submit"
						>
							Create
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
