import { XIconAttrPosInt } from "@/app/components/icons";
import { attrTypeInfo } from ".";

export const _posintAttrType: attrTypeInfo = {
	pretty_name: "Positive Integer",
	serialize_as: "PositiveInteger",
	icon: <XIconAttrPosInt />,
	extra_params: null,

	value_preview: (params) => params.attr.value,
	old_value: (params) => params.attr.value,
};
