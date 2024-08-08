import { Button, Select, Text, TextInput } from "@mantine/core";
import { TreeModal } from "../tree_modal";
import { useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { dsTypes } from "..";

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

	return {
		open,
		modal: (
			<TreeModal
				opened={opened}
				close={close}
				title="Create new dataset"
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
					data={dsTypes.map((x) => {
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
						onMouseDown={() => {
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
						}}
					>
						Create
					</Button>
				</Button.Group>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
					}}
				>
					<Text c="red">
						{errorMessage.response
							? errorMessage.response
							: errorMessage.name
							? errorMessage.name
							: errorMessage.type
							? errorMessage.type
							: ""}
					</Text>
				</div>
			</TreeModal>
		),
	};
}
