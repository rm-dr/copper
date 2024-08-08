import { XIconAttrBlob } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text } from "@mantine/core";
import { ppBytes } from "../ppbytes";

export const _blobAttrType: attrTypeInfo = {
	pretty_name: "Blob",
	serialize_as: "Blob",
	icon: <XIconAttrBlob />,
	extra_params: null,

	value_preview: (params) => (
		<Text c="dimmed" fs="italic">{`Blob ${params.attr.handle} (${ppBytes(
			params.attr.size,
		)})`}</Text>
	),
};
