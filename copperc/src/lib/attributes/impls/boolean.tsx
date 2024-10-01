import { attrTypeInfo } from "..";
import { ToggleRight } from "lucide-react";
import { BasicForm } from "../_basicform";

export const _booleanAttrType: attrTypeInfo = {
	pretty_name: "Boolean",
	serialize_as: "Boolean",
	icon: <ToggleRight />,
	create_params: {
		form: (params) => BasicForm({ attr_type: { type: "Boolean" }, ...params }),
	},
};
