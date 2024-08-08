import { XIconAttrBlob } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text } from "@mantine/core";
import { ppBytes } from "../ppbytes";
import Image from "next/image";
import { ReactNode } from "react";

export const _blobAttrType: attrTypeInfo = {
	pretty_name: "Blob",
	serialize_as: "Blob",
	icon: <XIconAttrBlob />,
	extra_params: null,

	value_preview: (params) => {
		if (params.attr.value === null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return (
				<Text c="dimmed" fs="italic">{`Blob ${params.attr.handle} (${ppBytes(
					params.attr.size,
				)})`}</Text>
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
					attr: params.attr_name,
				});

			if (params.attr_val.format.startsWith("image/")) {
				return <BlobPanelImage src={data_url} attr_val={params.attr_val} />;
			} else if (params.attr_val.format.startsWith("audio/")) {
				return <BlobPanelAudio src={data_url} attr_val={params.attr_val} />;
			} else {
				return (
					<BlobPanelUnknown
						src={data_url}
						icon={<XIconAttrBlob style={{ height: "5rem" }} />}
						attr_val={params.attr_val}
					/>
				);
			}
		},
	},
};

export function BlobPanelImage(params: { src: string; attr_val: any }) {
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

export function BlobPanelAudio(params: { src: string; attr_val: any }) {
	return (
		<audio controls>
			<source src={params.src} type={params.attr_val.format}></source>
		</audio>
	);
}

export function BlobPanelUnknown(params: {
	src: string;
	icon: ReactNode;
	attr_val: any;
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
				<Text>{ppBytes(params.attr_val.size)} of binary data</Text>{" "}
			</div>
			<div>
				<Text>Click to download.</Text>
			</div>
		</div>
	);
}
