import { ReactElement, ReactNode } from "react";
import { attrTypeInfo } from "..";
import { Binary, Trash2, Upload, X } from "lucide-react";
import { useMutation } from "@tanstack/react-query";
import { components } from "@/lib/api/openapi";
import { edgeclient } from "@/lib/api/client";
import { useForm, UseFormReturnType } from "@mantine/form";
import { AttrNameEntry, AttrSubmitButtons } from "../_basicform";
import { ActionIcon, Switch, Text } from "@mantine/core";
import { ppBytes } from "@/lib/ppbytes";
import Image from "next/image";

export const _blobAttrType: attrTypeInfo<"Blob"> = {
	pretty_name: "Blob",
	serialize_as: "Blob",
	icon: <Binary />,
	create_params: {
		form: (params) => BlobForm({ attr_type: { type: "Blob" }, ...params }),
	},

	table_cell: ({ value }) => {
		return (
			<div
				style={{
					paddingLeft: "0.5rem",
					overflow: "hidden",
					width: "100%",
					textOverflow: "ellipsis",
					whiteSpace: "nowrap",
					color: "var(--mantine-color-dimmed)",
					fontFamily: "monospace",
					fontStyle: "italic",
				}}
			>
				{value.mime}

				{value.size === null || value.size === undefined
					? " (??? bytes)"
					: ` (${ppBytes(value.size)})`}
			</div>
		);
	},

	editor: {
		type: "panel",

		panel_body: (params) => {
			return <BlobPanel {...params} />;
		},
	},
};

export function BlobPanel(params: {
	item_id: number;
	attr_id: number;
	value: {
		mime: string;
		size?: number | null;
		type: "Blob";
	};
	inner?: boolean;
}) {
	const data_url = `/api/item/${params.item_id}/attr/${params.attr_id}`;

	let inner: ReactNode | null = (
		<_PanelBodyUnknown src={data_url} icon={<X />} attr_value={params.value} />
	);

	if (params.value.mime != null && params.value.mime.startsWith("image/")) {
		inner = <_PanelBodyImage src={data_url} attr_value={params.value} />;
	} else if (
		params.value.mime != null &&
		params.value.mime.startsWith("audio/")
	) {
		inner = <_PanelBodyAudio src={data_url} attr_value={params.value} />;
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
			<div
				style={{
					width: "100%",
					flexGrow: 1,
					padding: params.inner !== true ? "0.5rem" : undefined,
					cursor: "zoom-in",
				}}
			>
				<a
					target="_blank"
					href={data_url}
					rel="noopener noreferrer"
					style={{ width: "100%", height: "100%", cursor: "inherit" }}
				>
					{inner}
				</a>
			</div>
			{params.inner !== true ? (
				<_PanelBottom attr_value={params.value} />
			) : null}
		</div>
	);
}

// Same as basicform, but with the "unique" switch hidden.
// It has no effect on blobs.
function BlobForm(params: {
	attr_type: { type: "Text" | "Boolean" | "Blob" };
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

	const form = useForm<{
		new_attr_name: string | null;
		is_unique: boolean;
		is_not_null: boolean;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_unique: false,
			is_not_null: false,
		},
		validate: {
			new_attr_name: (value) =>
				value === null
					? "Attribute name is required"
					: value.trim().length === 0
						? "Attribute name cannot be empty"
						: null,
		},
	});

	const reset = () => {
		form.reset();
		params.close();
	};

	return (
		<form
			onSubmit={form.onSubmit((values) => {
				if (values.new_attr_name === null) {
					// This is unreachable
					reset();
					return;
				}

				doCreate.mutate({
					data_type: params.attr_type,
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

				<Switch
					label="Not null"
					key={form.key("is_not_null")}
					disabled={doCreate.isPending}
					{...form.getInputProps("is_not_null")}
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

export function _PanelBodyImage(params: {
	src: string;
	attr_value: Extract<components["schemas"]["ItemAttrData"], { type: "Blob" }>;
}) {
	return (
		<div
			style={{
				position: "relative",
				width: "100%",
				height: "100%",
			}}
		>
			<Image
				alt=""
				src={params.src}
				fill
				style={{
					objectFit: "contain",
				}}
			/>
		</div>
	);
}

export function _PanelBottom(params: {
	attr_value: Extract<
		components["schemas"]["ItemAttrData"],
		{ type: "Blob" } | { type: "Binary" }
	>;
}) {
	return (
		<div
			style={{
				display: "flex",
				flexDirection: "row",
				alignItems: "center",
				width: "100%",
				gap: "0.5rem",
				backgroundColor: "var(--mantine-color-dark-6)",
				padding: "0.5rem",
			}}
		>
			<div>
				<Text>
					{params.attr_value.size === null ||
					params.attr_value.size === undefined
						? "??? bytes"
						: ppBytes(params.attr_value.size)}
				</Text>
			</div>
			<div style={{ flexGrow: 1 }}>
				<Text ff="monospace">{params.attr_value.mime}</Text>
			</div>
			<div>
				<ActionIcon variant="filled" color="red">
					<Trash2 />
				</ActionIcon>
			</div>
			<div>
				<ActionIcon variant="filled">
					<Upload />
				</ActionIcon>
			</div>
		</div>
	);
}

export function _PanelBodyAudio(params: {
	src: string;
	attr_value: Extract<
		components["schemas"]["ItemAttrData"],
		{ type: "Blob" } | { type: "Binary" }
	>;
}) {
	return (
		<audio controls>
			<source src={params.src} type={params.attr_value.mime as string}></source>
		</audio>
	);
}

export function _PanelBodyUnknown(params: {
	src: string;
	icon: ReactNode;
	attr_value: Extract<
		components["schemas"]["ItemAttrData"],
		{ type: "Blob" } | { type: "Binary" }
	>;
}) {
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
			<div>{params.icon}</div>
			<div>
				<Text>
					{params.attr_value.size === null ||
					params.attr_value.size === undefined
						? "??? bytes"
						: ppBytes(params.attr_value.size)}{" "}
					of binary data
				</Text>{" "}
			</div>
			<div>
				<Text>Click to download.</Text>
			</div>
		</div>
	);
}
