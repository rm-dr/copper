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
		if (params.attr_value.type !== "PositiveInteger") {
			return <>Unreachable!</>;
		}

		if (params.attr_value.value === null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return <Text>{params.attr_value.value}</Text>;
		}
	},

	editor: {
		type: "inline",
		old_value: (params) => {
			if (params.attr_value.type !== "PositiveInteger") {
				return <>Unreachable!</>;
			}

			if (params.attr_value.value === null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else {
				return <Text>{params.attr_value.value}</Text>;
			}
		},

		new_value: (params) => {
			if (params.attr_value.type !== "PositiveInteger") {
				return <>Unreachable!</>;
			}

			return (
				<NumberInput
					placeholder="empty value"
					allowDecimal={false}
					allowNegative={false}
					defaultValue={params.attr_value.value || undefined}
				/>
			);
		},
	},
};
