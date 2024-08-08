import { XIconAttrFloat } from "@/app/components/icons";
import { attrTypeInfo } from ".";
import { NumberInput } from "@mantine/core";

export const _floatAttrType: attrTypeInfo = {
	pretty_name: "Float",
	serialize_as: "Float",
	icon: <XIconAttrFloat />,
	extra_params: null,

	value_preview: (params) => params.attr.value,

	editor: {
		type: "inline",
		old_value: (params) => params.attr.value,

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
