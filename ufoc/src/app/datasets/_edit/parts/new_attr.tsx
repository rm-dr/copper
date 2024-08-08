import { XIconPlus, XIconX } from "@/app/components/icons";
import {
	ActionIcon,
	Button,
	Popover,
	Select,
	Text,
	TextInput,
} from "@mantine/core";
import { useState } from "react";
import { ButtonPopover } from "./popover";

export function NewAttrButton(params: {
	dataset_name: string;
	class_name: string;
	onSuccess: () => void;
}) {
	const [isLoading, setLoading] = useState(false);

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		type: string | null;
		response: string | null;
	}>({ name: null, type: null, response: null });

	const [opened, setOpened] = useState(false);
	const [newAttrName, setNewAttrName] = useState("");
	const [newAttrType, setNewAttrType] = useState<string | null>(null);

	return (
		<ButtonPopover
			color={"green"}
			icon={<XIconPlus style={{ width: "70%", height: "70%" }} />}
			isLoading={isLoading}
			isOpened={opened}
			setOpened={(opened) => {
				setOpened(opened);
				setLoading(false);
				setNewAttrName("");
				setErrorMessage({
					name: null,
					type: null,
					response: null,
				});
			}}
		>
			<TextInput
				placeholder="New attr name"
				size="sm"
				disabled={isLoading}
				error={errorMessage.name !== null}
				onChange={(e) => {
					setNewAttrName(e.currentTarget.value);
					setErrorMessage((m) => {
						return {
							...m,
							name: null,
						};
					});
				}}
			/>

			<Select
				size="sm"
				required={true}
				style={{ marginTop: "1rem" }}
				placeholder={"select attr type"}
				data={["Text", "Binary", "Blob", "Integer"]}
				error={errorMessage.type !== null}
				onChange={(val) => {
					setNewAttrType(val);
					setErrorMessage((m) => {
						return {
							...m,
							type: null,
						};
					});
				}}
				comboboxProps={{
					transitionProps: {
						transition: "fade-down",
						duration: 200,
					},
					withinPortal: false,
				}}
				clearable
			/>

			<div style={{ marginTop: "1rem" }}>
				<Button
					variant="filled"
					color={errorMessage === null ? "green" : "red"}
					fullWidth
					size="xs"
					leftSection={<XIconPlus />}
					onClick={() => {
						setLoading(true);
						if (newAttrName == "") {
							setLoading(false);
							setErrorMessage((m) => {
								return {
									...m,
									name: "Name cannot be empty",
								};
							});
							return;
						} else if (newAttrType === null) {
							setLoading(false);
							setErrorMessage((m) => {
								return {
									...m,
									type: "Type cannot be empty",
								};
							});
							return;
						}

						setErrorMessage({
							name: null,
							type: null,
							response: null,
						});
						fetch("/api/attr/add", {
							method: "POST",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify({
								class: params.class_name,
								dataset: params.dataset_name,
								attr: newAttrName,
								data_type: {
									type: newAttrType,
								},
								options: {
									unique: false,
								},
							}),
						}).then((res) => {
							setLoading(false);
							if (res.status == 400) {
								res.text().then((text) => {
									setErrorMessage((m) => {
										return {
											...m,
											response: text,
										};
									});
								});
							} else if (!res.ok) {
								res.text().then((text) => {
									setErrorMessage((m) => {
										return {
											...m,
											response: `Error ${res.status}: ${text}`,
										};
									});
								});
							} else {
								params.onSuccess();
								setLoading(false);
								setOpened(false);
							}
						});
					}}
				>
					Add attribute
				</Button>

				<Text c="red" ta="center">
					{errorMessage.response
						? errorMessage.response
						: errorMessage.name
						? errorMessage.name
						: errorMessage.type
						? errorMessage.type
						: ""}
				</Text>
			</div>
		</ButtonPopover>
	);
}
