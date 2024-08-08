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

export function NewAttrButton(params: {
	dataset_name: string;
	class_name: string;
	onSuccess: () => void;
}) {
	const [opened, setOpened] = useState(false);
	const [isLoading, setLoading] = useState(false);

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		type: string | null;
		response: string | null;
	}>({ name: null, type: null, response: null });

	const [newAttrName, setNewAttrName] = useState("");
	const [newAttrType, setNewAttrType] = useState<string | null>(null);

	const reset = () => {
		// Reset on change
		setLoading(false);
		setNewAttrName("");
		setErrorMessage({
			name: null,
			type: null,
			response: null,
		});
	};

	return (
		<Popover
			position="bottom"
			withArrow
			shadow="md"
			trapFocus
			width={"20rem"}
			opened={opened}
			onChange={(e) => {
				setOpened(e);
				reset();
			}}
		>
			<Popover.Target>
				<ActionIcon
					loading={isLoading}
					variant="light"
					color={opened ? "red" : "green"}
					style={{ cursor: "default" }}
					onClick={() => {
						setOpened((o) => !o);
						reset();
					}}
				>
					{opened ? (
						<XIconX style={{ width: "70%", height: "70%" }} />
					) : (
						<XIconPlus style={{ width: "70%", height: "70%" }} />
					)}
				</ActionIcon>
			</Popover.Target>
			<Popover.Dropdown>
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

							fetch(
								`/api/datasets/${params.dataset_name}/classes/${params.class_name}/attrs/${newAttrName}`,
								{
									method: "POST",
									headers: {
										"Content-Type": "application/json",
									},
									body: JSON.stringify({
										data_type: {
											type: newAttrType,
										},
										options: {
											unique: false,
										},
									}),
								},
							).then((res) => {
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
			</Popover.Dropdown>
		</Popover>
	);
}
