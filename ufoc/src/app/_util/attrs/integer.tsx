import { XIconAttrInt } from "@/app/components/icons";
import { attrTypeInfo } from ".";

export const _intAttrType: attrTypeInfo = {
	pretty_name: "Integer",
	serialize_as: "Integer",
	icon: <XIconAttrInt />,
	extra_params: null,

	value_preview: (params) => params.attr.value,
	old_value: (params) => params.attr.value,
};
