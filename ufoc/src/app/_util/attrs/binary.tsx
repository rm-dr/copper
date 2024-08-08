import { XIconAttrBinary } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text } from "@mantine/core";
import { ppBytes } from "../ppbytes";

export const _binaryAttrType: attrTypeInfo = {
	pretty_name: "Binary",
	serialize_as: "Binary",
	icon: <XIconAttrBinary />,
	extra_params: null,

	value_preview: (params) => (
		<Text c="dimmed" fs="italic">{`Binary (${ppBytes(
			params.attr.size,
		)})`}</Text>
	),
};
