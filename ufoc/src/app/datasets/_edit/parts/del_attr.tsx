import { XIconPlus, XIconTrash, XIconX } from "@/app/components/icons";
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

export function DeleteAttrButton(params: {
	dataset_name: string;
	class_name: string;
	attr_name: string;
	onSuccess: () => void;
}) {
	const [isLoading, setLoading] = useState(false);
	const [opened, setOpened] = useState(false);
	const [delAttrName, setDelAttrName] = useState("");

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		response: string | null;
	}>({ name: null, response: null });

	// This is run when we submit
	const del_attr = () => {
		if (delAttrName != params.attr_name) {
			setErrorMessage((e) => {
				return {
					...e,
					name: "Attribute name does not match",
				};
			});
			return;
		}
		setLoading(true);

		fetch("/api/attr/del", {
			method: "delete",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				class: params.class_name,
				dataset: params.dataset_name,
				attr: params.attr_name,
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
					<Text c="orange" span>{` ${params.attr_name} `}</Text>
					below to confirm.
				</Text>
			</div>

			<TextInput
				placeholder="Enter attribute name"
				size="sm"
				disabled={isLoading}
				error={errorMessage.name !== null}
				onChange={(e) => {
					setDelAttrName(e.currentTarget.value);
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
					onClick={del_attr}
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
