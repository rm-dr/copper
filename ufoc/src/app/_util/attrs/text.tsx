import { XIcon } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text, Textarea } from "@mantine/core";
import { IconLetterCase } from "@tabler/icons-react";

export const _textAttrType: attrTypeInfo = {
	pretty_name: "Text",
	serialize_as: "Text",
	icon: <XIcon icon={IconLetterCase} />,
	extra_params: null,

	value_preview: (params) => {
		if (params.attr.value == null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else if (params.attr.value == "") {
			return (
				<Text c="dimmed" fs="italic">
					empty string
				</Text>
			);
		} else if (params.attr.value.trim() == "") {
			return <Text c="dimmed">{`""`}</Text>;
		} else {
			return params.attr.value;
		}
	},

	editor: {
		type: "inline",

		old_value: (params) => {
			if (params.attr.value == null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else if (params.attr.value == "") {
				return (
					<Text c="dimmed" fs="italic">
						empty string
					</Text>
				);
			} else if (params.attr.value.trim() == "") {
				return <Text c="dimmed">{`""`}</Text>;
			} else {
				return params.attr.value;
			}
		},

		new_value: (params) => {
			return (
				<Textarea
					radius="0px"
					placeholder="no value"
					autosize
					minRows={1}
					defaultValue={params.attr.value}
					onChange={params.onChange}
				/>
			);
		},
	},
};
