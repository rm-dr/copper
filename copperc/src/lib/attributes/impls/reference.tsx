import { attrTypeInfo } from "..";
import { Ampersand } from "lucide-react";
import {
	AttrCommonOptions,
	AttrNameEntry,
	AttrSubmitButtons,
} from "../_basicform";
import { components } from "@/lib/api/openapi";
import { edgeclient } from "@/lib/api/client";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ReactElement } from "react";
import { Select } from "@mantine/core";
import { useForm, UseFormReturnType } from "@mantine/form";
import { BlobPanel } from "./blob";

export const _referenceAttrType: attrTypeInfo<"Reference"> = {
	pretty_name: "Reference",
	serialize_as: "Reference",
	icon: <Ampersand />,
	create_params: {
		form: (params) => Form(params),
	},

	table_cell: ({ dataset, value }) => {
		const c = dataset.classes.find((x) => x.id === value.class)!;

		return (
			<div
				style={{
					paddingLeft: "0.5rem",
					width: "100%",
					overflow: "hidden",
					textOverflow: "ellipsis",
					whiteSpace: "nowrap",
					color: "var(--mantine-color-dimmed)",
					fontStyle: "italic",
				}}
			>
				{`${c.name} #${value.item}`}
			</div>
		);
	},

	editor: {
		type: "panel",

		panel_body: (params) => {
			const pa = params.value.primary_attr;

			if (pa.type === "NotAvailable") {
				return (
					<div
						style={{
							display: "flex",
							flexDirection: "column",
							justifyContent: "center",
							alignItems: "center",

							paddingLeft: "0.5rem",
							width: "100%",
							height: "100%",

							color: "var(--mantine-color-dimmed)",
							fontStyle: "italic",
							fontWeight: 500,
							fontSize: "1.2rem",
							userSelect: "none",
						}}
					>
						No attribute available
					</div>
				);
			}

			if (pa.type === "Blob") {
				return (
					<BlobPanel
						item_id={params.value.item}
						attr_id={pa.attr}
						value={pa}
						inner={true}
					/>
				);
			}

			let v = <div>UNSET!</div>;
			if (pa.type === "Boolean") {
				v = pa.value ? (
					<span style={{ color: "var(--mantine-color-green-5)" }}>true</span>
				) : (
					<span style={{ color: "var(--mantine-color-red-5)" }}>false</span>
				);
			} else if (pa.type === "Float" || pa.type === "Integer") {
				v = <span>{pa.value}</span>;
			} else if (pa.type === "Hash") {
				v = <span style={{ fontFamily: "monospace" }}>{pa.value}</span>;
			} else if (pa.type === "Text") {
				v = <span>{pa.value}</span>;
			}

			return (
				<div
					style={{
						display: "flex",
						flexDirection: "column",
						justifyContent: "center",
						alignItems: "center",

						paddingLeft: "0.5rem",
						width: "100%",
						height: "100%",

						color: "var(--mantine-color-dimmed)",
						fontStyle: "italic",
						fontWeight: 500,
						fontSize: "1.2rem",
						userSelect: "none",
					}}
				>
					<div>{pa.type}</div>
					{v}
				</div>
			);
		},
	},
};

function Form(params: {
	dataset_id: number;
	class_id: number;
	onSuccess: () => void;
	close: () => void;
}): ReactElement {
	const doCreate = useMutation({
		mutationFn: async (body: components["schemas"]["NewAttributeRequest"]) => {
			return await edgeclient.POST("/class/{class_id}/attribute", {
				params: { path: { class_id: params.class_id } },
				body,
			});
		},

		onSuccess: async (res) => {
			if (res.response.status === 200) {
				reset();
				params.onSuccess();
			} else {
				throw new Error(res.error);
			}
		},

		onError: (err) => {
			throw err;
		},
	});

	const datasetinfo = useQuery({
		queryKey: ["dataset"],
		queryFn: async () => {
			const res = await edgeclient.GET("/dataset/{dataset_id}", {
				params: { path: { dataset_id: params.dataset_id } },
			});
			if (res.response.status === 401) {
				location.replace("/");
			}

			return res.data;
		},
	});

	const form = useForm<{
		new_attr_name: string | null;
		is_unique: boolean;
		is_not_null: boolean;
		reference_target_class: string | null;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_unique: false,
			is_not_null: false,
			reference_target_class: null,
		},
		validate: {
			new_attr_name: (value) =>
				value === null
					? "Attribute name is required"
					: value.trim().length === 0
						? "Attribute name cannot be empty"
						: null,

			reference_target_class: (value) =>
				value === null ? "Reference target is required" : null,
		},
	});

	const reset = () => {
		form.reset();
		params.close();
	};

	return (
		<form
			onSubmit={form.onSubmit((values) => {
				if (
					values.new_attr_name === null ||
					values.reference_target_class === null
				) {
					// This is unreachable
					reset();
					return;
				}

				// Parse value as an integer.
				// This should never fail, see select options.
				// Mantine select only supports string values.
				let c = null;
				try {
					c = parseInt(values.reference_target_class);
				} catch {}

				if (c === null) {
					return;
				}

				doCreate.mutate({
					data_type: {
						type: "Reference",
						class: c,
					},
					name: values.new_attr_name,
					options: {
						is_unique: values.is_unique,
						is_not_null: values.is_not_null,
					},
				});
			})}
		>
			<div
				style={{
					display: "flex",
					flexDirection: "column",
					gap: "0.5rem",
				}}
			>
				<AttrNameEntry
					form={form as UseFormReturnType<unknown>}
					isLoading={doCreate.isPending}
				/>

				<AttrCommonOptions
					form={form as UseFormReturnType<unknown>}
					isLoading={doCreate.isPending}
				/>

				<Select
					required={true}
					placeholder={"Select class"}
					data={
						datasetinfo.data === undefined
							? []
							: datasetinfo.data.classes.map((x) => ({
									label: x.name,
									value: x.id.toString(),
								}))
					}
					clearable
					disabled={doCreate.isPending}
					key={form.key("reference_target_class")}
					{...form.getInputProps("reference_target_class")}
				/>

				<AttrSubmitButtons
					form={form as UseFormReturnType<unknown>}
					errorMessage={doCreate.error === null ? null : doCreate.error.message}
					isLoading={doCreate.isPending}
					reset={reset}
				/>
			</div>
		</form>
	);
}

/*
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
			// Otherwise, show that attribute in this panel
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
					const shown_attr = Object.entries(i_data.attrs)
						.filter(
							// Do not try to show a reference inside a reference panel,
							// this could cause an infinite look
							//
							// (if that reference points to a reference that points to this reference)
							([a, v]) => v?.type !== "Reference",
						)
						.sort(
							([a_a, a_v], [b_a, b_v]) =>
								(a_v as unknown as components["schemas"]["ItemListData"]).attr
									.idx -
								(b_v as unknown as components["schemas"]["ItemListData"]).attr
									.idx,
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

function RefForm(params: {
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
	close: () => void;
}) {
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm<{
		new_attr_name: string | null;
		reference_target_class: string | null;
		is_unique: boolean;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			reference_target_class: null,
			is_unique: false,
		},
		validate: {
			new_attr_name: (value) =>
				value === null
					? "Attribute name is required"
					: value.trim().length === 0
						? "Attribute name cannot be empty"
						: null,

			reference_target_class: (value) =>
				value === null ? "Reference target is required" : null,
		},
	});

	const reset = () => {
		form.reset();
		setLoading(false);
		setErrorMessage(null);
		params.close();
	};

	return (
		<form
			onSubmit={form.onSubmit((values) => {
				setLoading(true);
				setErrorMessage(null);

				if (
					values.reference_target_class === null ||
					values.new_attr_name === null
				) {
					// This is unreachable
					params.close();
					return;
				}

				APIclient.POST("/attr/add", {
					body: {
						class: params.class.handle,
						dataset: params.dataset_name,
						new_attr_name: values.new_attr_name,
						data_type: {
							type: "Reference",
							class: parseInt(values.reference_target_class),
						},
						options: {
							unique: values.is_unique,
						},
					},
				}).then(({ data, error }) => {
					setLoading(false);
					if (error !== undefined) {
						setErrorMessage(error);
					} else {
						setLoading(false);
						params.close();
					}
				});
			})}
		>
			<div
				style={{
					display: "flex",
					flexDirection: "column",
					gap: "0.5rem",
				}}
			>
				<AttrNameEntry form={form} isLoading={isLoading} />

				<ClassSelector
					selectedDataset={params.dataset_name}
					onSelect={(_) => { }}
					form={{
						form,
						key: "reference_target_class",
					}}
				/>

				<AttrCommonOptions form={form} isLoading={isLoading} />

				<AttrSubmitButtons
					form={form}
					errorMessage={errorMessage}
					isLoading={isLoading}
					reset={reset}
				/>
			</div>
		</form>
	);
}


*/
