import { XIconAttrInt } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { NumberInput } from "@mantine/core";

export const _intAttrType: attrTypeInfo = {
	pretty_name: "Integer",
	serialize_as: "Integer",
	icon: <XIconAttrInt />,
	extra_params: null,

	value_preview: (params) => params.attr.value,

	editor: {
		type: "inline",
		old_value: (params) => params.attr.value,

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
