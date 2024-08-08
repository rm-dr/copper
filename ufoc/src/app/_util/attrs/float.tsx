import { IconDecimal } from "@tabler/icons-react";
import { attrTypeInfo } from ".";
import { NumberInput, Text } from "@mantine/core";
import { XIcon } from "@/app/components/icons";

export const _floatAttrType: attrTypeInfo = {
	pretty_name: "Float",
	serialize_as: "Float",
	icon: <XIcon icon={IconDecimal} />,
	extra_params: null,

	value_preview: (params) => {
		if (params.attr.value === null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return params.attr.value;
		}
	},

	editor: {
		type: "inline",
		old_value: (params) => {
			if (params.attr.value === null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else {
				return params.attr.value;
			}
		},

		new_value: (params) => {
			return (
				<NumberInput
					placeholder="empty value"
					allowDecimal={true}
					defaultValue={params.attr.value}
				/>
			);
		},
	},
};
