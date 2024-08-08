import { XIconAttrBinary } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text } from "@mantine/core";
import { ppBytes } from "../ppbytes";
import { BlobPanelAudio, BlobPanelImage, BlobPanelUnknown } from "./blob";

export const _binaryAttrType: attrTypeInfo = {
	pretty_name: "Binary",
	serialize_as: "Binary",
	icon: <XIconAttrBinary />,
	extra_params: null,

	value_preview: (params) => {
		if (params.attr.size === null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return (
				<Text c="dimmed" fs="italic">{`Binary (${ppBytes(
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

			if (params.attr_val.mime.startsWith("image/")) {
				return <BlobPanelImage src={data_url} attr_val={params.attr_val} />;
			} else if (params.attr_val.mime.startsWith("audio/")) {
				return <BlobPanelAudio src={data_url} attr_val={params.attr_val} />;
			} else {
				return (
					<BlobPanelUnknown
						src={data_url}
						icon={<XIconAttrBinary style={{ height: "5rem" }} />}
						attr_val={params.attr_val}
					/>
				);
			}
		},
	},
};
