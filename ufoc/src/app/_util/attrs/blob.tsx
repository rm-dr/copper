import { attrTypeInfo } from ".";
import { ActionIcon, Text } from "@mantine/core";
import { ppBytes } from "../ppbytes";
import Image from "next/image";
import { ReactNode } from "react";
import { XIcon } from "@/app/components/icons";
import { IconFileDigit, IconTrash, IconUpload } from "@tabler/icons-react";
import { components } from "../api/openapi";

export const _blobAttrType: attrTypeInfo = {
	pretty_name: "Blob",
	serialize_as: "Blob",
	icon: <XIcon icon={IconFileDigit} />,
	extra_params: null,

	value_preview: (params) => {
		if (params.attr_value.type !== "Blob") {
			return <>Unreachable!</>;
		}

		if (params.attr_value.size == null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return (
				<Text c="dimmed" fs="italic">{`Blob ${
					params.attr_value.handle
				} (${ppBytes(params.attr_value.size)})`}</Text>
			);
		}
	},

	editor: {
		type: "panel",

		panel_body: (params) => {
			const data_url =
				"/api/item/attr?" +
				new URLSearchParams({
					dataset: params.dataset,
					class: params.class,
					item_idx: params.item_idx.toString(),
					attr: params.attr_value.attr.handle.toString(),
				});

			if (params.attr_value.type !== "Blob") {
				return <>Unreachable!</>;
			}

			if (
				params.attr_value.mime != null &&
				params.attr_value.mime.startsWith("image/")
			) {
				return (
					<_PanelBodyImage src={data_url} attr_value={params.attr_value} />
				);
			} else if (
				params.attr_value.mime != null &&
				params.attr_value.mime.startsWith("audio/")
			) {
				return (
					<_PanelBodyAudio src={data_url} attr_value={params.attr_value} />
				);
			} else {
				return (
					<_PanelBodyUnknown
						src={data_url}
						icon={<XIcon icon={IconFileDigit} style={{ height: "5rem" }} />}
						attr_value={params.attr_value}
					/>
				);
			}
		},

		panel_bottom: (params) => {
			if (params.attr_value.type !== "Blob") {
				return <>Unreachable!</>;
			}

			return <_PanelBottom attr_value={params.attr_value} />;
		},
	},
};

export function _PanelBodyImage(params: {
	src: string;
	attr_value: Extract<
		components["schemas"]["ItemListData"],
		{ type: "Blob" } | { type: "Binary" }
	>;
}) {
	return (
		<Image
			alt=""
			src={params.src}
			fill
			style={{
				objectFit: "contain",
			}}
		/>
	);
}

export function _PanelBottom(params: {
	attr_value: Extract<
		components["schemas"]["ItemListData"],
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
				height: "100%",
				gap: "0.5rem",
			}}
		>
			<div>
				<Text>{ppBytes(params.attr_value.size || 0)}</Text>
			</div>
			<div style={{ flexGrow: 1 }}>
				<Text ff="monospace">{params.attr_value.mime}</Text>
			</div>
			<div>
				<ActionIcon variant="filled" color="red">
					<XIcon icon={IconTrash} style={{ width: "70%", height: "70%" }} />
				</ActionIcon>
			</div>
			<div>
				<ActionIcon variant="filled">
					<XIcon icon={IconUpload} style={{ width: "70%", height: "70%" }} />
				</ActionIcon>
			</div>
		</div>
	);
}

export function _PanelBodyAudio(params: {
	src: string;
	attr_value: Extract<
		components["schemas"]["ItemListData"],
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
		components["schemas"]["ItemListData"],
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
				<Text>{ppBytes(params.attr_value.size as number)} of binary data</Text>{" "}
			</div>
			<div>
				<Text>Click to download.</Text>
			</div>
		</div>
	);
}
