import { XIcon } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { NumberInput, Text } from "@mantine/core";
import { IconHexagonPlus } from "@tabler/icons-react";

export const _posintAttrType: attrTypeInfo = {
	pretty_name: "Positive Integer",
	serialize_as: "PositiveInteger",
	icon: <XIcon icon={IconHexagonPlus} />,
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
					allowNegative={false}
					defaultValue={params.attr.value}
				/>
			);
		},
	},
};
