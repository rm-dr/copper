import { XIcon } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { Text, Textarea } from "@mantine/core";
import { IconLetterCase } from "@tabler/icons-react";
import { BaseForm } from "./helpers/baseform";

export const _textAttrType: attrTypeInfo = {
	pretty_name: "Text",
	serialize_as: "Text",
	icon: <XIcon icon={IconLetterCase} />,
	params: {
		form: (params) => BaseForm({ attr_type: { type: "Text" }, ...params }),
	},

	value_preview: (params) => {
		if (params.attr_value.type !== "Text") {
			return <>Unreachable!</>;
		}

		if (params.attr_value.value == null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else if (params.attr_value.value == "") {
			return (
				<Text c="dimmed" fs="italic">
					empty string
				</Text>
			);
		} else if (params.attr_value.value.trim() == "") {
			return (
				<>
					<Text c="dimmed" span>{`"`}</Text>
					<Text span>{params.attr_value.value}</Text>
					<Text c="dimmed" span>{`"`}</Text>;
				</>
			);
		} else {
			return <Text>{params.attr_value.value}</Text>;
		}
	},

	editor: {
		type: "inline",

		old_value: (params) => {
			if (params.attr_value.type !== "Text") {
				return <>Unreachable!</>;
			}

			if (params.attr_value.value == null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else if (params.attr_value.value == "") {
				return (
					<Text c="dimmed" fs="italic">
						empty string
					</Text>
				);
			} else if (params.attr_value.value.trim() == "") {
				return <Text c="dimmed">{`""`}</Text>;
			} else {
				return <Text>{params.attr_value.value}</Text>;
			}
		},

		new_value: (params) => {
			if (params.attr_value.type !== "Text") {
				return <>Unreachable!</>;
			}

			return (
				<Textarea
					radius="0px"
					placeholder="no value"
					autosize
					minRows={1}
					defaultValue={params.attr_value.value || undefined}
					onChange={params.onChange}
				/>
			);
		},
	},
};
