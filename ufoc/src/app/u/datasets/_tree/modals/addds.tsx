import { Button, Select, Text, TextInput } from "@mantine/core";
import { useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBase } from "@/app/components/modal_base";
import { datasetTypes } from "@/app/_util/datasets";
import { XIcon } from "@/app/components/icons";
import { IconDatabasePlus } from "@tabler/icons-react";
import { APIclient } from "@/app/_util/api";

export function useNewDsModal(onSuccess: () => void) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm<{
		name: null | string;
		params: {
			type: any;
		};
	}>({
		mode: "uncontrolled",
		initialValues: {
			name: "",
			params: {
				type: null,
			},
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

						if (values.name === null) {
							throw Error(
								"Entered unreachable code: name is null, this should've been caught by `validate`",
							);
						}

						APIclient.POST("/dataset/add", {
							body: {
								name: values.name,
								params: {
									// TODO: clean up when we have more than one dataset type
									type: values.params.type as unknown as "Local",
								},
							},
						}).then(({ data, error }) => {
							setLoading(false);
							if (error !== undefined) {
								setErrorMessage(error);
							}

							onSuccess();
							reset();
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
							onClick={reset}
							disabled={isLoading}
						>
							Cancel
						</Button>
						<Button
							variant="filled"
							fullWidth
							color="green"
							loading={isLoading}
							leftSection={<XIcon icon={IconDatabasePlus} />}
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
