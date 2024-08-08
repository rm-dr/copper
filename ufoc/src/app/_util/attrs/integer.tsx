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
		if (params.attr_value.type !== "Integer") {
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
			if (params.attr_value.type !== "Integer") {
				return <>Unreachable!</>;
			}

			if (params.attr_value.value === null) {
				return (
					<Text key={params.key} c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else {
				return <Text key={params.key}>{params.attr_value.value}</Text>;
			}
		},

		new_value: (params) => {
			if (params.attr_value.type !== "Integer") {
				return <>Unreachable!</>;
			}

			return (
				<NumberInput
					key={params.key}
					placeholder="empty value"
					allowDecimal={false}
					allowNegative={true}
					defaultValue={params.attr_value.value || undefined}
				/>
			);
		},
	},
};
