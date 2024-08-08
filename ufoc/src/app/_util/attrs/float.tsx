import { XIconAttrFloat } from "@/app/components/icons";
import { attrTypeInfo } from ".";

export const _floatAttrType: attrTypeInfo = {
	pretty_name: "Float",
	serialize_as: "Float",
	icon: <XIconAttrFloat />,
	extra_params: null,

	value_preview: (params) => params.attr.value,
	old_value: (params) => params.attr.value,
};
