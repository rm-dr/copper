import { Button, Select, Text, TextInput } from "@mantine/core";
import { TreeModal } from "../tree_modal";
import { useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { datasetTypes } from "..";
import { XIconDatabasePlus } from "@/app/components/icons";

export function useNewDsModal(onSuccess: () => void) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		type: string | null;
		response: string | null;
	}>({ name: null, type: null, response: null });

	const [newDsName, setNewDsName] = useState("");
	const [newDsType, setNewDsType] = useState<null | string>(null);

	const new_ds = () => {
		setLoading(true);
		if (newDsName == "" || newDsName === null) {
			setLoading(false);
			setErrorMessage((e) => ({
				...e,
				name: "Name cannot be empty",
			}));
			return;
		} else if (newDsType === null) {
			setLoading(false);
			setErrorMessage((e) => ({
				...e,
				type: "Dataset type is required",
			}));
			return;
		}

		setErrorMessage((e) => ({
			name: null,
			type: null,
			response: null,
		}));

		fetch(`/api/dataset/add`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				name: newDsName,
				params: {
					type: newDsType,
				},
			}),
		}).then((res) => {
			setLoading(false);
			if (!res.ok) {
				res.text().then((text) => {
					setErrorMessage((e) => ({
						...e,
						response: text,
					}));
				});
			} else {
				// Successfully created new dataset
				onSuccess();
				close();
			}
		});
	};

	return {
		open,
		modal: (
			<TreeModal
				opened={opened}
				close={() => {
					// Reset everything on close
					setNewDsName("");
					setNewDsType(null);
					setLoading(false);
					setErrorMessage({ name: null, type: null, response: null });
					close();
				}}
				title="Create a new dataset"
				keepOpen={isLoading}
			>
				<TextInput
					data-autofocus
					placeholder="enter dataset name"
					required={true}
					disabled={isLoading}
					error={errorMessage.name !== null}
					onChange={(e) => {
						if (errorMessage.name !== null) {
							setErrorMessage((e) => ({
								...e,
								name: null,
							}));
						}
						setNewDsName(e.currentTarget.value);
					}}
				/>
				<Select
					required={true}
					style={{ marginTop: "1rem" }}
					placeholder="select dataset type"
					data={datasetTypes.map((x) => {
						return x.pretty_name;
					})}
					error={errorMessage.type !== null}
					onChange={(value, _option) => {
						if (errorMessage.type !== null) {
							setErrorMessage((e) => ({
								...e,
								type: null,
							}));
						}
						setNewDsType(value);
					}}
					disabled={isLoading}
					comboboxProps={{
						transitionProps: { transition: "fade-down", duration: 200 },
					}}
					clearable
				/>

				<Button.Group style={{ marginTop: "1rem" }}>
					<Button
						variant="light"
						fullWidth
						color="red"
						onMouseDown={close}
						disabled={isLoading}
					>
						Cancel
					</Button>
					<Button
						variant="filled"
						fullWidth
						color="green"
						loading={isLoading}
						onMouseDown={new_ds}
						leftSection={<XIconDatabasePlus />}
					>
						Create
					</Button>
				</Button.Group>

				<Text c="red" ta="center">
					{errorMessage.response
						? errorMessage.response
						: errorMessage.name
						? errorMessage.name
						: errorMessage.type
						? errorMessage.type
						: ""}
				</Text>
			</TreeModal>
		),
	};
}
