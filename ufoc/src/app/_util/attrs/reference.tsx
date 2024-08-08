import { attrTypeInfo, attrTypes } from ".";
import { ActionIcon, Loader, Text } from "@mantine/core";
import { ClassSelector } from "@/app/components/apiselect/class";
import {
	IconAmpersand,
	IconEdit,
	IconQuestionMark,
	IconTrash,
	IconX,
} from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { APIclient } from "../api";
import { components } from "../api/openapi";
import { useEffect, useState } from "react";

export const _refAttrType: attrTypeInfo = {
	pretty_name: "Reference",
	serialize_as: "Reference",
	icon: <XIcon icon={IconAmpersand} />,
	extra_params: {
		inputs_ok: checkRef,
		node: RefParams,
	},

	value_preview: (params) => {
		if (params.attr_value.type !== "Reference") {
			return <>Unreachable!</>;
		}

		if (params.attr_value.item === null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return (
				<Text c="dimmed">
					Reference to{" "}
					<Text c="dimmed" fs="italic" span>
						{params.attr_value.class}
					</Text>
				</Text>
			);
		}
	},

	editor: {
		type: "panel",

		panel_body: (params) => {
			// TODO: show "empty" in row

			return (
				<>
					<RefPanelBody
						dataset={params.dataset}
						class={params.class}
						item_idx={params.item_idx}
						attr_value={params.attr_value}
					/>
				</>
			);
		},

		panel_bottom: (params) => {
			if (
				params.attr_value.type !== "Reference" ||
				params.attr_value.item === null ||
				params.attr_value.item === undefined
			) {
				return <>Unreachable!</>;
			}

			// TODO: body and bottom in same fn?

			return (
				<div
					style={{
						display: "flex",
						flexDirection: "row",
						alignItems: "center",
						width: "100%",
						height: "100%",
						gap: "0.5rem",
					}}
				>
					<div style={{ flexGrow: 1 }}>
						<Text c="dimmed" span>
							Class:
						</Text>{" "}
						<Text span>{params.attr_value.class}</Text>
					</div>
					<div style={{ flexGrow: 1 }}>
						{params.attr_value.item === undefined ? (
							<Text c="dimmed" span>
								Empty reference
							</Text>
						) : (
							<>
								<Text c="dimmed" span>
									Item:
								</Text>{" "}
								<Text span>{params.attr_value.item.toString()}</Text>
							</>
						)}
					</div>
					<div style={{ flexGrow: 1 }}>
						<Text c="dimmed" span>
							Showing Attribute:
						</Text>{" "}
						<Text span>qq</Text>
					</div>
					<div>
						<ActionIcon variant="filled" color="red">
							<XIcon icon={IconTrash} style={{ width: "70%", height: "70%" }} />
						</ActionIcon>
					</div>
					<div>
						<ActionIcon variant="filled">
							<XIcon icon={IconEdit} style={{ width: "70%", height: "70%" }} />
						</ActionIcon>
					</div>
				</div>
			);
		},
	},
};

function RefPanelBody(params: {
	dataset: string;
	class: number;
	item_idx: number;
	attr_value: components["schemas"]["ItemListData"];
}) {
	const [data, setData] = useState<
		| {
				loading: true;
				error: null;
				data: null;
		  }
		| {
				loading: false;
				error: string;
				data: null;
		  }
		| {
				loading: false;
				error: null;
				data: components["schemas"]["ItemListItem"];
		  }
	>({
		loading: true,
		error: null,
		data: null,
	});

	useEffect(() => {
		if (
			params.attr_value.type !== "Reference" ||
			params.attr_value.item === null ||
			params.attr_value.item === undefined
		) {
			return;
		}

		console.log({
			dataset: params.dataset,
			class: params.attr_value.class,
			item: params.attr_value.item,
		});
		APIclient.GET("/item/get", {
			params: {
				query: {
					dataset: params.dataset,
					class: params.attr_value.class,
					item: params.attr_value.item,
				},
			},
		}).then(({ data, error }) => {
			if (error !== undefined) {
				setData({ loading: false, error, data: null });
			} else {
				setData({ loading: false, error: null, data });
			}
		});
	}, [params.attr_value, params.dataset]);

	if (
		params.attr_value.type !== "Reference" ||
		params.attr_value.item === null ||
		params.attr_value.item === undefined
	) {
		return <>Unreachable!</>;
	}

	let body;
	if (data.loading) {
		body = (
			<>
				<div>
					<Loader color="dimmed" size="4rem" />
				</div>
				<div>Loading..</div>
			</>
		);
	} else if (data.error !== null) {
		body = (
			<>
				<div>
					<XIcon
						icon={IconX}
						style={{ height: "5rem", color: "var(--mantine-color-red-7)" }}
					/>
				</div>
				<div>Error: {data.error}</div>
			</>
		);
	} else if (params.attr_value.item === undefined) {
		// TODO: show "empty" in row
		// (Don't show panel)
		body = (
			<>
				<div>
					<XIcon icon={IconAmpersand} style={{ height: "5rem" }} />
				</div>
				<div>No item selected</div>
			</>
		);
	} else {
		const first_attr = Object.entries(data.data?.attrs).sort(
			([aa, av], [ba, bv]) =>
				(av as unknown as components["schemas"]["ItemListData"]).attr.idx -
				(bv as unknown as components["schemas"]["ItemListData"]).attr.idx,
		)[0][1];
		if (first_attr === undefined) {
			body = (
				<>
					<div>
						<XIcon icon={IconQuestionMark} style={{ height: "5rem" }} />
					</div>
					<div>
						<Text span>{params.class}</Text>{" "}
						<Text c="dimmed" span>
							has no attributes.
						</Text>
					</div>
				</>
			);
		} else {
			const d = attrTypes.find((x) => {
				return x.serialize_as === first_attr.attr.data_type.type;
			});

			const attr_value = data.data.attrs[first_attr.attr.handle.toString()];
			if (attr_value === undefined) {
				return <>Unreachable</>;
			}

			if (d?.editor.type === "panel") {
				body = d.editor.panel_body({
					dataset: params.dataset,
					class: first_attr.attr.class as number,
					item_idx: params.attr_value.item as number,
					attr_value,
				});
			} else if (d?.editor.type == "inline") {
				body = "TODO";
			}
		}
	}

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				justifyContent: "center",
				alignItems: "center",
				color: "var(--mantine-color-dimmed)",
				height: "100%",
			}}
		>
			{body}
		</div>
	);
}

function checkRef(params: {
	state: any;
	setErrorMessage: (message: null | any) => void;
}): boolean {
	if (params.state === null) {
		params.setErrorMessage("Reference target is required");
		return false;
	} else if (params.state.class === null) {
		params.setErrorMessage("Reference target is required");
		return false;
	}

	return true;
}

function RefParams(params: {
	onChange: (state: null | any) => void;
	dataset_name: string;
	setErrorMessage: (message: null | any) => void;
	errorMessage: null | any;
}) {
	return (
		<ClassSelector
			selectedDataset={params.dataset_name}
			onSelect={(v) => {
				if (v === null) {
					params.onChange({ class: null });
				} else {
					params.onChange({ class: v });
				}
			}}
		/>
	);
}
