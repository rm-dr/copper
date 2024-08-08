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
		if (params.attr_value.type !== "Float") {
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
			if (params.attr_value.type !== "Float") {
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
			if (params.attr_value.type !== "Float") {
				return <>Unreachable!</>;
			}

			return (
				<NumberInput
					key={params.key}
					placeholder="empty value"
					allowDecimal={true}
					defaultValue={params.attr_value.value || undefined}
				/>
			);
		},
	},
};
