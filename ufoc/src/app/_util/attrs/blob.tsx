import { XIconAttrBlob } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text } from "@mantine/core";
import { ppBytes } from "../ppbytes";

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

	editor: { type: "panel" },
};
