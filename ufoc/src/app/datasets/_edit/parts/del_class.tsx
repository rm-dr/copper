import { XIconTrash } from "@/app/components/icons";
import { Button, Text, TextInput } from "@mantine/core";
import { useState } from "react";
import { ButtonPopover } from "../../_util/popover";

export function DeleteClassButton(params: {
	dataset_name: string;
	class_name: string;
	onSuccess: () => void;
}) {
	const [isLoading, setLoading] = useState(false);
	const [opened, setOpened] = useState(false);
	const [delClassName, setDelClassName] = useState("");

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		response: string | null;
	}>({ name: null, response: null });

	// This is run when we submit
	const del_class = () => {
		if (delClassName != params.class_name) {
			setErrorMessage((e) => {
				return {
					...e,
					name: "Class name does not match",
				};
			});
			return;
		}
		setLoading(true);

		fetch("/api/class/del", {
			method: "delete",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				class: params.class_name,
				dataset: params.dataset_name,
			}),
		})
			.then((res) => {
				setLoading(false);
				if (!res.ok) {
					res.text().then((text) => {
						setErrorMessage((e) => {
							return {
								...e,
								response: text,
							};
						});
					});
				} else {
					params.onSuccess();
					setOpened(false);
				}
			})
			.catch((err) => {
				setLoading(false);
				setErrorMessage((e) => {
					return {
						...e,
						response: `Error: ${err}`,
					};
				});
			});
	};

	return (
		<ButtonPopover
			color={"red"}
			icon={<XIconTrash style={{ width: "70%", height: "70%" }} />}
			isLoading={isLoading}
			isOpened={opened}
			setOpened={(opened) => {
				setOpened(opened);
				setLoading(false);
				setErrorMessage({
					name: null,
					response: null,
				});
			}}
		>
			<div
				style={{
					marginBottom: "1rem",
				}}
			>
				<Text c="red" size="sm">
					This action will irreversably destroy data. Enter
					<Text c="orange" span>{` ${params.class_name} `}</Text>
					below to confirm.
				</Text>
			</div>

			<TextInput
				placeholder="Enter class name"
				size="sm"
				disabled={isLoading}
				error={errorMessage.name !== null}
				onChange={(e) => {
					setDelClassName(e.currentTarget.value);
					setErrorMessage((m) => {
						return {
							...m,
							name: null,
						};
					});
				}}
			/>

			<div style={{ marginTop: "1rem" }}>
				<Button
					variant="filled"
					color="red"
					fullWidth
					size="xs"
					leftSection={<XIconTrash />}
					onClick={del_class}
				>
					Confirm
				</Button>

				<Text c="red" ta="center">
					{errorMessage.response
						? errorMessage.response
						: errorMessage.name
						? errorMessage.name
						: ""}
				</Text>
			</div>
		</ButtonPopover>
	);
}
