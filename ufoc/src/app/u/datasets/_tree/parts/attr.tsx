import {
	XIconDots,
	XIconEdit,
	XIconRow,
	XIconTrash,
} from "@/app/components/icons";
import { ActionIcon, Button, Menu, Text, TextInput, rem } from "@mantine/core";
import { TreeEntry } from "../tree_entry";
import { useDisclosure } from "@mantine/hooks";
import { useState } from "react";
import { TreeModal } from "../tree_modal";
import { attrTypes } from "../attrs";

export function AttrList(params: {
	update_tree: () => void;
	dataset: string;
	class: string;
	open: boolean;

	attrs: {
		name: string;
		type: string;
	}[];
}) {
	return (
		<div
			style={{
				paddingLeft: "2rem",
				transition: "200ms",
				display: params.open ? "" : "none",
			}}
		>
			{params.attrs.map(({ name: attr_name, type: attr_type }) => {
				// Find attr icon
				let type_def = attrTypes.find((x) => {
					return x.serialize_as === attr_type;
				});

				return (
					<TreeEntry
						key={`dataset-${params.dataset}-class-${params.class}-attr-${attr_type}`}
						is_clickable={true}
						is_selected={false}
						onClick={() => {}}
						icon={type_def?.icon}
						text={attr_name}
						icon_tooltip={type_def?.pretty_name}
						icon_tooltip_position={"left"}
						right={
							<AttrMenu
								dataset_name={params.dataset}
								class_name={params.class}
								attr_name={attr_name}
								onSuccess={params.update_tree}
								disabled={!params.open}
							/>
						}
					/>
				);
			})}
		</div>
	);
}

function AttrMenu(params: {
	dataset_name: string;
	class_name: string;
	attr_name: string;
	disabled: boolean;
	onSuccess: () => void;
}) {
	const { open: openDelAttr, modal: modalDelAttr } = useDeleteAttrModal({
		dataset_name: params.dataset_name,
		class_name: params.class_name,
		attr_name: params.attr_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelAttr}
			<Menu
				shadow="md"
				position="right-start"
				withArrow
				arrowPosition="center"
				disabled={params.disabled}
			>
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIconDots style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Attribute</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconEdit style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Rename
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openDelAttr}
					>
						Delete this attribute
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}

export function useDeleteAttrModal(params: {
	dataset_name: string;
	class_name: string;
	attr_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [isLoading, setLoading] = useState(false);
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
					close();
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

	return {
		open,
		modal: (
			<TreeModal
				opened={opened}
				close={() => {
					// Reset everything on close
					setDelAttrName("");
					setLoading(false);
					setErrorMessage({ name: null, response: null });
					close();
				}}
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

				<TextInput
					data-autofocus
					placeholder="Enter attribute name"
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
						color="red"
						fullWidth
						leftSection={<XIconTrash />}
						onClick={del_attr}
					>
						Confirm
					</Button>
				</Button.Group>

				<Text c="red" ta="center">
					{errorMessage.response
						? errorMessage.response
						: errorMessage.name
						? errorMessage.name
						: ""}
				</Text>
			</TreeModal>
		),
	};
}
