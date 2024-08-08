import {
	XIconDots,
	XIconEdit,
	XIconFolder,
	XIconPlus,
	XIconTrash,
} from "@/app/components/icons";
import {
	ActionIcon,
	Button,
	Group,
	Menu,
	Select,
	SelectProps,
	Text,
	TextInput,
	rem,
} from "@mantine/core";
import { Dispatch, SetStateAction, useState } from "react";
import { attrTypes } from "../attrs";
import { AttrList } from "./attr";
import { TreeEntry } from "../tree_entry";
import { useDisclosure } from "@mantine/hooks";
import { TreeModal } from "../tree_modal";
import { TreeData } from "..";

export function ClassList(params: {
	update_tree: () => void;
	open: boolean;
	setTreeData: Dispatch<SetStateAction<TreeData>>;
	dataset_name: string;
	dataset_idx: number;
	classes: {
		name: string;
		open: boolean;
		attrs: {
			name: string;
			type: string;
		}[];
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
			{params.classes.map(
				({ name: class_name, open: class_open, attrs }, idx) => {
					return (
						<div key={`dataset-${params.dataset_name}-class-${class_name}`}>
							<TreeEntry
								is_clickable={true}
								is_selected={class_open}
								onClick={() => {
									params.setTreeData((x) => {
										let t = { ...x };
										if (t.datasets !== null) {
											t.datasets[params.dataset_idx].classes[idx].open =
												!class_open;
										}
										return t;
									});
								}}
								icon={<XIconFolder />}
								text={class_name}
								expanded={class_open}
								right={
									<ClassMenu
										dataset_name={params.dataset_name}
										class_name={class_name}
										onSuccess={params.update_tree}
										disabled={!params.open}
									/>
								}
							/>
							<AttrList
								update_tree={params.update_tree}
								open={class_open}
								dataset={params.dataset_name}
								class={class_name}
								attrs={attrs}
							/>
						</div>
					);
				},
			)}
		</div>
	);
}

function ClassMenu(params: {
	dataset_name: string;
	class_name: string;
	disabled: boolean;
	onSuccess: () => void;
}) {
	const { open: openDelete, modal: modalDelete } = useDeleteClassModal({
		dataset_name: params.dataset_name,
		class_name: params.class_name,
		onSuccess: params.onSuccess,
	});

	const { open: openAddAttr, modal: modalAddAttr } = useAddAttrModal({
		dataset_name: params.dataset_name,
		class_name: params.class_name,
		onSuccess: params.onSuccess,
	});

	return (
		<>
			{modalDelete}
			{modalAddAttr}
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
					<Menu.Label>Class</Menu.Label>
					<Menu.Item
						leftSection={
							<XIconEdit style={{ width: rem(14), height: rem(14) }} />
						}
					>
						Rename
					</Menu.Item>
					<Menu.Item
						leftSection={
							<XIconPlus style={{ width: rem(14), height: rem(14) }} />
						}
						onClick={openAddAttr}
					>
						Add attribute
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
						Delete this class
					</Menu.Item>
				</Menu.Dropdown>
			</Menu>
		</>
	);
}

export function useDeleteClassModal(params: {
	dataset_name: string;
	class_name: string;
	onSuccess: () => void;
}) {
	const [isLoading, setLoading] = useState(false);
	const [opened, { open, close }] = useDisclosure(false);
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
					setDelClassName("");
					setLoading(false);
					setErrorMessage({ name: null, response: null });
					close();
				}}
				title="Delete class"
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
						<Text c="orange" span>{` ${params.class_name} `}</Text>
						below to confirm.
					</Text>
				</div>

				<TextInput
					data-autofocus
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
						color="red"
						loading={isLoading}
						leftSection={<XIconTrash />}
						onClick={del_class}
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

export function useAddAttrModal(params: {
	dataset_name: string;
	class_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [isLoading, setLoading] = useState(false);

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		type: string | null;
		response: string | null;
		extra_params: null | any;
	}>({ name: null, type: null, response: null, extra_params: null });

	const [newAttrName, setNewAttrName] = useState("");
	const [newAttrType, setNewAttrType] = useState<string | null>(null);

	// This is an object set by an attributes's "extra params" node.
	// This is expanded directly into the new attr POST, see `add_attr` below.
	const [newAttrParams, setNewAttrParams] = useState<null | Object>(null);

	// Get input ui for attr-specific parameters
	let NewAttrParamsInput: null | any = null;
	let newAttrExtraParams: null | any = null;
	if (newAttrType !== null) {
		const d = attrTypes.find((x) => {
			return x.serialize_as === newAttrType;
		});
		if (d !== undefined && d.extra_params !== null) {
			// This is a function, but DON'T RUN IT!
			// It's a react component that is placed into tsx below.
			NewAttrParamsInput = d.extra_params.node;
			newAttrExtraParams = d.extra_params;
		}
	}

	const add_attr = () => {
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
		} else if (newAttrExtraParams !== null) {
			if (
				!newAttrExtraParams.inputs_ok({
					state: newAttrParams,
					setErrorMessage: (m: any) => {
						setErrorMessage((e) => ({ ...e, extra_params: m }));
					},
				})
			) {
				setLoading(false);
				return;
			}
		}

		setErrorMessage({
			name: null,
			type: null,
			response: null,
			extra_params: null,
		});

		let extra_params = {};
		if (newAttrParams !== null) {
			extra_params = newAttrParams;
		}

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
					...extra_params,
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
				close();
			}
		});
	};

	const renderSelectOption: SelectProps["renderOption"] = ({
		option,
		checked,
	}) => {
		let icon = null;
		const d = attrTypes.find((x) => {
			return x.serialize_as === option.value;
		});
		if (d !== undefined) {
			icon = d.icon;
		}
		return (
			<Group flex="1" gap="xs">
				<div
					style={{
						// center icon vertically
						display: "flex",
						flexDirection: "column",
						justifyContent: "center",
						alignItems: "center",
						height: "100%",
						// looks
						width: "1.5rem",
						color: "var(--mantine-color-dimmed)",
					}}
				>
					{icon}
				</div>

				{option.label}
			</Group>
		);
	};

	return {
		open,
		modal: (
			<TreeModal
				opened={opened}
				close={() => {
					// Reset everything on close
					setNewAttrName("");
					setNewAttrType(null);
					setNewAttrParams(null);
					setErrorMessage({
						name: null,
						type: null,
						response: null,
						extra_params: null,
					});
					close();
				}}
				title="Add an attribute"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						Add an attribute to the class
						<Text c="gray" span>{` ${params.class_name}`}</Text>:
					</Text>
				</div>
				<TextInput
					data-autofocus
					placeholder="New attr name"
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
					renderOption={renderSelectOption}
					required={true}
					style={{ marginTop: "1rem" }}
					placeholder={"select attr type"}
					data={attrTypes.map((x) => ({
						label: x.pretty_name,
						value: x.serialize_as,
						disabled: false,
					}))}
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
					}}
					clearable
				/>

				{NewAttrParamsInput === null ? null : (
					<div style={{ marginTop: "1rem", marginBottom: "1rem" }}>
						<NewAttrParamsInput
							dataset_name={params.dataset_name}
							errorMessage={errorMessage.extra_params}
							setErrorMessage={(m: any) => {
								setErrorMessage((e) => ({ ...e, extra_params: m }));
							}}
							onChange={(x: any) => {
								setNewAttrParams(x);
							}}
						/>
					</div>
				)}

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
						color={
							Object.values(errorMessage).every((x) => x === null)
								? "green"
								: "red"
						}
						fullWidth
						leftSection={<XIconPlus />}
						onClick={add_attr}
					>
						Create Attribute
					</Button>
				</Button.Group>

				<Text c="red" ta="center">
					{/* TODO: this is ugly */}
					{errorMessage.response
						? errorMessage.response
						: errorMessage.name
							? errorMessage.name
							: errorMessage.type
								? errorMessage.type
								: errorMessage.extra_params
									? errorMessage.extra_params
									: ""}
				</Text>
			</TreeModal>
		),
	};
}
