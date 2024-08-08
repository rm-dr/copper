import { XIcon } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { NumberInput, Text } from "@mantine/core";
import { IconHexagon3 } from "@tabler/icons-react";

export const _intAttrType: attrTypeInfo = {
	pretty_name: "Integer",
	serialize_as: "Integer",
	icon: <XIcon icon={IconHexagon3} />,
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
					allowDecimal={false}
					allowNegative={true}
					defaultValue={params.attr.value}
				/>
			);
		},
	},
};
