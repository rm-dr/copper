import {
	XIconDots,
	XIconEdit,
	XIconFolderPlus,
	XIconTrash,
} from "@/app/components/icons";
import { ActionIcon, Button, Menu, Text, TextInput, rem } from "@mantine/core";

import { TreeData, datasetTypes } from "..";
import { Dispatch, SetStateAction, useState } from "react";

import styles from "../tree.module.scss";
import { ClassList } from "./class";
import { TreeEntry } from "../tree_entry";
import { useDisclosure } from "@mantine/hooks";
import { TreeModal } from "../tree_modal";

export function DatasetList(params: {
	update_tree: () => void;
	datasets: {
		name: string;
		type: string;
		open: boolean;
		classes: {
			name: string;
			open: boolean;
			attrs: {
				name: string;
				type: string;
			}[];
		}[];
	}[];
	setTreeData: Dispatch<SetStateAction<TreeData>>;
}) {
	return (
		<div className={styles.dataset_list}>
			{params.datasets.map(
				(
					{
						name: dataset_name,
						type: dataset_type,
						open: dataset_open,
						classes,
					},
					idx,
				) => {
					// Find dataset icon
					let d = datasetTypes.find((x) => {
						return x.serialize_as === dataset_type;
					});
					let icon;
					if (d === undefined) {
						icon = <></>;
					} else {
						icon = d.icon;
					}

					return (
						<div
							key={`dataset-${dataset_name}`}
							style={{
								paddingLeft: "0",
								transition: "200ms",
							}}
						>
							<TreeEntry
								is_clickable={true}
								is_selected={dataset_open}
								onClick={() => {
									params.setTreeData((x) => {
										let t = { ...x };
										if (t.datasets !== null) {
											t.datasets[idx].open = !dataset_open;
										}
										return t;
									});
								}}
								icon={icon}
								icon_text={dataset_name}
								left_width={"6rem"}
								text={""}
								expanded={dataset_open}
								right={
									<DatasetMenu
										dataset_name={dataset_name}
										onSuccess={params.update_tree}
									/>
								}
							/>
							<ClassList
								open={dataset_open}
								update_tree={params.update_tree}
								setTreeData={params.setTreeData}
								dataset_name={dataset_name}
								dataset_idx={idx}
								classes={classes}
							/>
						</div>
					);
				},
			)}
		</div>
	);
}

function DatasetMenu(params: { dataset_name: string; onSuccess: () => void }) {
	const { open: openDelete, modal: modalDelete } = useDeleteDatasetModal({
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	const { open: openAddClass, modal: modalAddClass } = useAddClassModal({
		dataset_name: params.dataset_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalAddClass}
			<Menu shadow="md" position="right-start" withArrow arrowPosition="center">
				<Menu.Target>
					<ActionIcon color="gray" variant="subtle" size={"2rem"} radius={"0"}>
						<XIconDots style={{ width: "70%", height: "70%" }} />
					</ActionIcon>
				</Menu.Target>

				<Menu.Dropdown>
					<Menu.Label>Dataset</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconEdit style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIconFolderPlus style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openAddClass}
					>
						Add class
					</Menu.Item>
					<Menu.Divider />

					<Menu.Label>Danger zone</Menu.Label>
					<Menu.Item
						color="red"
						leftSection={
							<XIconTrash style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openDelete}
					>
						Delete this dataset
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}

export function useDeleteDatasetModal(params: {
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [isLoading, setLoading] = useState(false);
	const [opened, { open, close }] = useDisclosure(false);
	const [delDatasetName, setDelDatasetName] = useState("");

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		response: string | null;
	}>({ name: null, response: null });

	// This is run when we submit
	const del_dataset = () => {
		if (delDatasetName != params.dataset_name) {
			setErrorMessage((e) => {
				return {
					...e,
					name: "Dataset name does not match",
				};
			});
			return;
		}
		setLoading(true);

		fetch("/api/dataset/del", {
			method: "delete",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				dataset_name: params.dataset_name,
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
					close();
					setErrorMessage({ name: null, response: null });
				}}
				title="Delete dataset"
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
						<Text c="orange" span>{` ${params.dataset_name} `}</Text>
						below to confirm.
					</Text>
				</div>

				<TextInput
					data-autofocus
					placeholder="Enter dataset name"
					disabled={isLoading}
					error={errorMessage.name !== null}
					onChange={(e) => {
						setDelDatasetName(e.currentTarget.value);
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
						fullWidth
						leftSection={<XIconTrash />}
						color="red"
						loading={isLoading}
						onMouseDown={del_dataset}
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

export function useAddClassModal(params: {
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [newClassName, setNewClassName] = useState("");

	const new_class = () => {
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
					close();
				}
			})
			.catch((e) => {
				setLoading(false);
				setErrorMessage(`Error: ${e}`);
			});
	};

	return {
		open,
		modal: (
			<TreeModal
				opened={opened}
				close={() => {
					close();
					setErrorMessage(null);
				}}
				title="Add a class"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="teal" size="sm">
						Add a class to the dataset
						<Text c="lime" span>{` ${params.dataset_name}`}</Text>:
					</Text>
				</div>
				<TextInput
					data-autofocus
					placeholder="New class name"
					disabled={isLoading}
					error={errorMessage !== null}
					onChange={(e) => {
						setNewClassName(e.currentTarget.value);
						setErrorMessage(null);
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
						fullWidth
						color={errorMessage === null ? "green" : "red"}
						loading={isLoading}
						leftSection={<XIconFolderPlus />}
						onMouseDown={new_class}
					>
						Create class
					</Button>
				</Button.Group>
				<Text c="red" ta="center">
					{errorMessage ? errorMessage : ""}
				</Text>
			</TreeModal>
		),
	};
}
