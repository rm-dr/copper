import styles from "./edit.module.scss";
import { Panel, PanelSection } from "../../components/panel";

import {
	XIconDatabase,
	XIconDatabaseX,
	XIconEdit,
	XIconFolder,
	XIconFolderX,
	XIconFolders,
	XIconRow,
	XIconServer,
	XIconSettings,
	XIconTrash,
} from "@/app/components/icons";
import {
	ActionIcon,
	Button,
	Loader,
	Popover,
	Text,
	TextInput,
} from "@mantine/core";
import { ReactNode, useEffect, useState } from "react";
import { Slider } from "../_util/slider";
import { NewClassButton } from "./parts/new_class";
import { NewAttrButton } from "./parts/new_attr";
import { DeleteAttrButton } from "./parts/del_attr";
import { DeleteClassButton } from "./parts/del_class";

const Wrapper = (params: { children: ReactNode }) => {
	return (
		<div
			style={{
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				width: "100%",
				marginTop: "1rem",
				marginBottom: "1rem",
			}}
		>
			<div
				style={{
					display: "block",
					textAlign: "center",
				}}
			>
				{params.children}
			</div>
		</div>
	);
};

type DatasetDetails = {
	error: boolean;
	loading: boolean;

	name: null | string;
	classes:
		| null
		| {
				handle: number;
				name: string;

				attrs: {
					handle: number;
					name: string;

					data_type: {
						type: string;
					};
				}[];
		  }[];
};

export function useEdit(selected_dataset: string | null) {
	const [datasetDetails, setDatasetDetails] = useState<DatasetDetails>({
		error: false,
		loading: false,
		name: null,
		classes: null,
	});

	const update_ds_details = (dataset_name: string | null) => {
		setDatasetDetails({
			error: false,
			loading: true,
			name: null,
			classes: null,
		});

		if (dataset_name === null) {
			setDatasetDetails({
				error: false,
				loading: false,
				name: null,
				classes: null,
			});
			return;
		}

		fetch(
			"/api/class/list?" +
				new URLSearchParams({
					dataset: dataset_name,
				}).toString(),
		)
			.then((res) => res.json())
			.then((data) => {
				setDatasetDetails({
					error: false,
					loading: false,
					name: dataset_name,
					classes: data,
				});
			})
			.catch(() => {
				setDatasetDetails({
					error: true,
					loading: false,
					name: dataset_name,
					classes: [],
				});
			});
	};

	useEffect(() => {
		update_ds_details(selected_dataset);
	}, [selected_dataset]);

	var inner;
	if (datasetDetails.loading) {
		inner = (
			<Wrapper>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
						height: "5rem",
					}}
				>
					<Loader color="dimmed" size="3rem" />
				</div>
				<Text size="lg" c="dimmed">
					Loading...
				</Text>
			</Wrapper>
		);
	} else if (
		datasetDetails.classes === null ||
		datasetDetails.name === null ||
		selected_dataset === null
	) {
		inner = (
			<Wrapper>
				<XIconDatabaseX
					style={{
						height: "5rem",
						color: "var(--mantine-color-dimmed)",
					}}
				/>
				<Text size="lg" c="dimmed">
					No dataset selected
				</Text>
			</Wrapper>
		);
	} else {
		inner = (
			<>
				<PanelSection icon={<XIconSettings />} title={"General"}>
					<Slider
						icon={<XIconServer />}
						icon_text={"Local"}
						text={datasetDetails.name}
						right={<></>}
						is_clickable={false}
						is_selected={true}
					/>

					<NewClassButton
						dataset_name={selected_dataset}
						onSuccess={() => {
							update_ds_details(selected_dataset);
						}}
					/>
				</PanelSection>
				<PanelSection icon={<XIconFolders />} title={"Item classes"}>
					<div className={styles.itemclass_list}>
						<ItemClassList
							dataset_details={datasetDetails}
							onUpdate={() => {
								update_ds_details(selected_dataset);
							}}
						/>
					</div>
				</PanelSection>
			</>
		);
	}

	return (
		<>
			<Panel
				panel_id={styles.panel_id_edit}
				icon={<XIconDatabase />}
				title={"Edit dataset"}
			>
				{inner}
			</Panel>
		</>
	);
}

function RenameClassButton(params: { class_name: string }) {
	return (
		<Popover position="bottom" withArrow shadow="md" trapFocus width={"20rem"}>
			<Popover.Target>
				<ActionIcon
					variant="light"
					aria-label="Rename this class"
					color="blue"
					onMouseDown={console.log}
				>
					<XIconEdit style={{ width: "70%", height: "70%" }} />
				</ActionIcon>
			</Popover.Target>
			<Popover.Dropdown>
				<Text size="sm" c="blue">
					You are renaming
					<Text c="cyan" span>{` ${params.class_name}`}</Text>. Enter new name
					below.
				</Text>

				<TextInput placeholder="New class name" size="sm" />

				<div style={{ marginTop: "1rem" }}>
					<Button
						variant="filled"
						color="blue"
						fullWidth
						size="xs"
						leftSection={<XIconEdit />}
					>
						Rename
					</Button>
				</div>
			</Popover.Dropdown>
		</Popover>
	);
}

function RenameAttrButton(params: { class_name: string; attr_name: string }) {
	return (
		<Popover position="bottom" withArrow shadow="md" trapFocus width={"20rem"}>
			<Popover.Target>
				<ActionIcon
					variant="light"
					aria-label="Rename this attribute"
					color="blue"
					onMouseDown={console.log}
				>
					<XIconEdit style={{ width: "70%", height: "70%" }} />
				</ActionIcon>
			</Popover.Target>
			<Popover.Dropdown>
				<Text size="sm" c="blue">
					You are renaming
					<Text c="cyan" span>{` ${params.attr_name}`}</Text>. Enter new name
					below.
				</Text>

				<TextInput placeholder="New attr name" size="sm" />

				<div style={{ marginTop: "1rem" }}>
					<Button
						variant="filled"
						color="blue"
						fullWidth
						size="xs"
						leftSection={<XIconEdit />}
					>
						Rename
					</Button>
				</div>
			</Popover.Dropdown>
		</Popover>
	);
}

export function ItemClassList(params: {
	dataset_details: DatasetDetails;
	onUpdate: () => void;
}) {
	if (
		params.dataset_details.classes === null ||
		params.dataset_details.name === null ||
		params.dataset_details.loading
	) {
		console.error("Entered uneachable code");
		return <></>;
	}

	if (params.dataset_details.classes.length === 0) {
		return (
			<Wrapper>
				<XIconFolderX
					style={{
						height: "5rem",
						color: "var(--mantine-color-dimmed)",
					}}
				/>
				<Text size="lg" c="dimmed">
					No item classes
				</Text>
			</Wrapper>
		);
	}

	return (
		<>
			{params.dataset_details.classes.map(({ name, handle, attrs }, idx) => {
				return (
					<div key={`itemclass-${name}`}>
						<Slider
							icon={<XIconFolder />}
							icon_text={"Item class"}
							text={name}
							is_selected={false}
							is_clickable={true}
							right={
								<>
									<NewAttrButton
										dataset_name={
											// TODO: fix this type check
											params.dataset_details.name === null
												? "unreachable"
												: params.dataset_details.name
										}
										class_name={name}
										onSuccess={params.onUpdate}
									/>
									<RenameClassButton class_name={name} />
									<DeleteClassButton
										dataset_name={
											// TODO: fix this type check
											params.dataset_details.name === null
												? "unreachable"
												: params.dataset_details.name
										}
										class_name={name}
										onSuccess={params.onUpdate}
									/>
								</>
							}
						/>
						<div style={{ marginLeft: "2rem" }}>
							{attrs.map(({ name: attr_name }) => {
								return (
									<Slider
										key={`attr-${name}-${attr_name}`}
										icon={<XIconRow />}
										icon_text={"Attribute"}
										text={attr_name}
										is_selected={false}
										is_clickable={true}
										right={
											<>
												<RenameAttrButton
													class_name={name}
													attr_name={attr_name}
												/>
												<DeleteAttrButton
													dataset_name={
														// TODO: fix this type check
														params.dataset_details.name === null
															? "unreachable"
															: params.dataset_details.name
													}
													class_name={name}
													attr_name={attr_name}
													onSuccess={params.onUpdate}
												/>
											</>
										}
									/>
								);
							})}
						</div>
					</div>
				);
			})}
		</>
	);
}
