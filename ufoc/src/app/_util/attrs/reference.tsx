import { attrTypeInfo, attrTypes } from ".";
import { Center, Loader, Text } from "@mantine/core";
import { ClassSelector } from "@/app/components/apiselect/class";
import { IconAmpersand, IconQuestionMark, IconX } from "@tabler/icons-react";
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

		return (
			<RefPanelPreview
				dataset={params.dataset}
				item_idx={params.item_idx}
				attr_value={params.attr_value}
			/>
		);
	},

	editor: {
		type: "panel",

		panel_body: (params) => {
			// TODO: show "empty" in row
			if (params.attr_value.type !== "Reference") {
				return <>Unreachable!</>;
			}

			return (
				<RefPanel
					dataset={params.dataset}
					item_idx={params.item_idx}
					attr_value={params.attr_value}
					inner={params.inner}
				/>
			);
		},
	},
};

function RefPanelPreview(params: {
	dataset: string;
	item_idx: number;
	attr_value: Extract<
		components["schemas"]["ItemListData"],
		{ type: "Reference" }
	>;
}) {
	const [data, setData] = useState<RefPanelData>({
		loading: true,
		error: null,
		data: null,
		class: null,
		shown_attr: null,
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

		Promise.all([
			APIclient.GET("/item/get", {
				params: {
					query: {
						dataset: params.dataset,
						class: params.attr_value.class,
						item: params.attr_value.item,
					},
				},
			}),
			APIclient.GET("/class/get", {
				params: {
					query: {
						dataset: params.dataset,
						class: params.attr_value.class,
					},
				},
			}),
		]).then(
			([
				{ data: i_data, error: i_error },
				{ data: c_data, error: c_error },
			]) => {
				if (i_error !== undefined) {
					setData({
						loading: false,
						error: i_error,
						data: null,
						class: null,
						shown_attr: null,
					});
				} else if (c_error !== undefined) {
					setData({
						loading: false,
						error: c_error,
						data: null,
						class: null,
						shown_attr: null,
					});
				} else {
					const shown_attr = Object.entries(i_data.attrs).sort(
						([aa, av], [ba, bv]) =>
							(av as unknown as components["schemas"]["ItemListData"]).attr
								.idx -
							(bv as unknown as components["schemas"]["ItemListData"]).attr.idx,
					)[0][1];
					setData({
						loading: false,
						error: null,
						data: i_data,
						class: c_data,
						shown_attr,
					});
				}
			},
		);
	}, [params.attr_value, params.dataset]);

	if (
		params.attr_value.type !== "Reference" ||
		params.attr_value.item === null ||
		params.attr_value.item === undefined
	) {
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
				Reference to
				{` #${params.attr_value.item} of `}
				<Text c="dimmed" fs="italic" span>
					{data.class?.name}
				</Text>
			</Text>
		);
	}
}

function RefPanelBody(params: {
	dataset: string;
	data: RefPanelData;
	ref_attr_value: Extract<
		components["schemas"]["ItemListData"],
		{ type: "Reference" }
	>;
}) {
	if (params.data.loading) {
		return (
			<>
				<div>
					<Loader color="dimmed" size="4rem" />
				</div>
				<div>Loading...</div>
			</>
		);
	} else if (params.data.error !== null) {
		return (
			<>
				<div>
					<XIcon
						icon={IconX}
						style={{ height: "5rem", color: "var(--mantine-color-red-7)" }}
					/>
				</div>
				<div>Error: {params.data.error}</div>
			</>
		);
	} else if (params.ref_attr_value.item === null) {
		return (
			<>
				<div>
					<XIcon icon={IconAmpersand} style={{ height: "5rem" }} />
				</div>
				<div>No item selected</div>
			</>
		);
	} else if (params.data.shown_attr === undefined) {
		return (
			<>
				<div>
					<XIcon icon={IconQuestionMark} style={{ height: "5rem" }} />
				</div>
				<div>
					<Text span>{params.data.class.name}</Text>{" "}
					<Text c="dimmed" span>
						has no attributes.
					</Text>
				</div>
			</>
		);
	} else {
		const d = attrTypes.find((x) => {
			return x.serialize_as === params.data.shown_attr?.attr.data_type.type;
		});

		const attr_value =
			params.data.data.attrs[params.data.shown_attr.attr.handle.toString()];
		if (attr_value === undefined) {
			return <>Unreachable</>;
		}

		if (d?.editor.type === "panel") {
			return d.editor.panel_body({
				dataset: params.dataset,
				item_idx: params.ref_attr_value.item as number,
				attr_value,
				inner: true,
			});
		} else if (d?.editor.type == "inline") {
			return (
				<div
					style={{
						overflowY: "scroll",
						whiteSpace: "pre-line",
						textWrap: "pretty",
						overflowWrap: "anywhere",
						width: "100%",
						background: "var(--mantine-color-dark-6)",
						padding: "0.5rem",
					}}
				>
					{d.editor.old_value({
						dataset: params.dataset,
						item_idx: params.ref_attr_value.item as number,
						attr_value,
					})}
				</div>
			);
		}
	}
}

function RefPanelBottom(params: {
	dataset: string;
	data: RefPanelData;
	ref_attr_value: Extract<
		components["schemas"]["ItemListData"],
		{ type: "Reference" }
	>;
}) {
	if (
		params.data.error !== null ||
		params.data.loading ||
		params.data.shown_attr === undefined
	) {
		return <></>;
	}

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				alignItems: "flex-start",
				width: "100%",
				gap: "0.5rem",
				backgroundColor: "var(--mantine-color-dark-6)",
				padding: "0.5rem",
			}}
		>
			<div
				style={{
					display: "flex",
					flexDirection: "row",
					alignItems: "center",
					width: "100%",
					gap: "0.5rem",
				}}
			>
				<div style={{ flexGrow: 1 }}>
					<Text c="dimmed" span>
						Class:
					</Text>{" "}
					<Text span>{params.data.class.name}</Text>
				</div>
				<div style={{ flexGrow: 1 }}>
					{params.ref_attr_value.item === null ||
					params.ref_attr_value.item === undefined ? (
						<Text c="dimmed" span>
							Empty reference
						</Text>
					) : (
						<>
							<Text c="dimmed" span>
								Item:
							</Text>{" "}
							<Text span>{params.ref_attr_value.item.toString()}</Text>
						</>
					)}
				</div>
			</div>
			<div>
				<Text c="dimmed" span>
					Showing Attribute:
				</Text>{" "}
				<Text span>
					{
						params.data.data.attrs[
							params.data.shown_attr.attr.handle.toString()
						]?.attr.name
					}
				</Text>
			</div>
		</div>
	);
}

type RefPanelData =
	| {
			loading: true;
			error: null;
			data: null;
			class: null;
			shown_attr: null;
	  }
	| {
			loading: false;
			error: string;
			data: null;
			class: null;
			shown_attr: null;
	  }
	| {
			loading: false;
			error: null;
			data: components["schemas"]["ItemListItem"];
			class: components["schemas"]["ClassInfo"];
			shown_attr: components["schemas"]["ItemListData"] | undefined;
	  };

function RefPanel(params: {
	dataset: string;
	item_idx: number;
	attr_value: Extract<
		components["schemas"]["ItemListData"],
		{ type: "Reference" }
	>;
	inner?: boolean;
}) {
	const [data, setData] = useState<RefPanelData>({
		loading: true,
		error: null,
		data: null,
		class: null,
		shown_attr: null,
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

		Promise.all([
			APIclient.GET("/item/get", {
				params: {
					query: {
						dataset: params.dataset,
						class: params.attr_value.class,
						item: params.attr_value.item,
					},
				},
			}),
			APIclient.GET("/class/get", {
				params: {
					query: {
						dataset: params.dataset,
						class: params.attr_value.class,
					},
				},
			}),
		]).then(
			([
				{ data: i_data, error: i_error },
				{ data: c_data, error: c_error },
			]) => {
				if (i_error !== undefined) {
					setData({
						loading: false,
						error: i_error,
						data: null,
						class: null,
						shown_attr: null,
					});
				} else if (c_error !== undefined) {
					setData({
						loading: false,
						error: c_error,
						data: null,
						class: null,
						shown_attr: null,
					});
				} else {
					const shown_attr = Object.entries(i_data.attrs).sort(
						([aa, av], [ba, bv]) =>
							(av as unknown as components["schemas"]["ItemListData"]).attr
								.idx -
							(bv as unknown as components["schemas"]["ItemListData"]).attr.idx,
					)[0][1];
					setData({
						loading: false,
						error: null,
						data: i_data,
						class: c_data,
						shown_attr,
					});
				}
			},
		);
	}, [params.attr_value, params.dataset]);

	if (
		params.attr_value.type !== "Reference" ||
		params.attr_value.item === null ||
		params.attr_value.item === undefined
	) {
		return <>Unreachable!</>;
	}

	return (
		<div
			style={{
				height: "100%",
				width: "100%",
				display: "flex",
				flexDirection: "column",
			}}
		>
			<div style={{ padding: "0.5rem", flexGrow: 1 }}>
				<RefPanelBody
					dataset={params.dataset}
					data={data}
					ref_attr_value={params.attr_value}
				/>
			</div>
			<RefPanelBottom
				dataset={params.dataset}
				data={data}
				ref_attr_value={params.attr_value}
			/>
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
