import { XIconFilePlus, XIconFolderPlus, XIconX } from "@/app/components/icons";
import { Button, Popover, Text, TextInput } from "@mantine/core";
import { useState } from "react";

export function NewClassButton(params: {
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, setOpened] = useState(false);
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [newClassName, setNewClassName] = useState("");

	return (
		<Popover
			position="bottom"
			withArrow
			shadow="md"
			trapFocus
			width={"target"}
			opened={opened}
			onChange={(e) => {
				setOpened(e);
				// Reset on change
				setLoading(false);
				setErrorMessage(null);
				setNewClassName("");
			}}
		>
			<Popover.Target>
				<Button
					onClick={() => {
						setOpened((o) => !o);
						// Reset on change
						setLoading(false);
						setErrorMessage(null);
						setNewClassName("");
					}}
					radius="0"
					loading={isLoading}
					variant="light"
					color={opened ? "red" : "green"}
					fullWidth
					leftSection={opened ? <XIconX /> : <XIconFolderPlus />}
					style={{ cursor: "default" }}
				>
					{opened ? "Cancel" : "Create a new item class"}
				</Button>
			</Popover.Target>
			<Popover.Dropdown>
				<TextInput
					placeholder="New class name"
					size="sm"
					disabled={isLoading}
					error={errorMessage !== null}
					onChange={(e) => {
						setNewClassName(e.currentTarget.value);
						setErrorMessage(null);
					}}
				/>
				<div style={{ marginTop: "1rem" }}>
					<Button
						variant="filled"
						color={errorMessage === null ? "green" : "red"}
						fullWidth
						size="xs"
						leftSection={<XIconFolderPlus />}
						onClick={() => {
							setLoading(true);
							if (newClassName == "") {
								setLoading(false);
								setErrorMessage("Name cannot be empty");
								return;
							}
							setErrorMessage(null);

							fetch(`/api/class/add`, {
								method: "POST",
								headers: {
									"Content-Type": "application/json",
								},
								body: JSON.stringify({
									class: newClassName,
									dataset: params.dataset_name,
								}),
							})
								.then((res) => {
									setLoading(false);
									if (!res.ok) {
										res.text().then((text) => {
											setErrorMessage(text);
										});
									} else {
										params.onSuccess();
										setOpened(false);
									}
								})
								.catch((e) => {
									setLoading(false);
									setErrorMessage(`Error: ${e}`);
								});
						}}
					>
						Create new class
					</Button>

					<Text c="red" ta="center">
						{errorMessage ? errorMessage : ""}
					</Text>
				</div>
			</Popover.Dropdown>
		</Popover>
	);
}
